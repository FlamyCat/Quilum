<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import EventCard from "$lib/components/EventCard.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";
    import Slot from "$lib/components/Slot.svelte";
    import {
        ChevronLeft,
        ChevronRight,
        Circle,
        CalendarPlus,
        CopyPlus,
    } from "@lucide/svelte";
    import {
        week_timetable,
        update_task,
        getKeyString,
        type Task,
        type Slot as ApiSlot,
        type SlotWithTasks,
    } from "$lib/api";

    function getWeekStart(date: Date): Date {
        const d = new Date(date);
        const day = d.getDay();
        const diff = d.getDate() - day + (day === 0 ? -6 : 1);
        d.setDate(diff);
        // Normalize to midnight to ensure correct day boundaries
        d.setHours(0, 0, 0, 0);
        return d;
    }

    const today = new Date();
    let weekStart = $state(getWeekStart(today));

    function formatDateISO(date: Date): string {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, "0");
        const day = String(date.getDate()).padStart(2, "0");
        return `${year}-${month}-${day}`;
    }

    function goToPrevWeek() {
        const d = new Date(weekStart);
        d.setDate(d.getDate() - 7);
        weekStart = d;
    }

    function goToNextWeek() {
        const d = new Date(weekStart);
        d.setDate(d.getDate() + 7);
        weekStart = d;
    }

    function goToCurrentWeek() {
        weekStart = getWeekStart(today);
    }

    let weekDays = $derived(
        Array.from({ length: 7 }, (_, i) => {
            const d = new Date(weekStart);
            d.setDate(d.getDate() + i);
            return d;
        }),
    );

    function formatDay(date: Date): string {
        return date.getDate().toString();
    }

    function formatMonth(date: Date): string {
        return date.toLocaleString("ru-RU", { month: "short" }).toLowerCase();
    }

    function isToday(date: Date): boolean {
        return (
            date.getDate() === today.getDate() &&
            date.getMonth() === today.getMonth() &&
            date.getFullYear() === today.getFullYear()
        );
    }

    interface CalendarEvent {
        type: "event";
        id: string;
        title: string;
        description?: string;
        displayStart: Date | null;
        displayEnd: Date | null;
        startedBefore: boolean;
        href: string;
    }

    interface CalendarSlot {
        type: "slot";
        id: string;
        slot: { starts_at: number; ends_at: number };
        tasks: [Task, number][];
        displayStart: Date;
        displayEnd: Date;
        startedBefore: boolean;
        endsAfter: boolean;
        href: string;
    }

    type CalendarDayItem = CalendarEvent | CalendarSlot;

    let events = $state<CalendarEvent[]>([]);
    let slotsWithTasks = $state<SlotWithTasks[]>([]);
    let loading = $state(true);

    async function loadWeekData() {
        loading = true;
        try {
            const weekStartISO = formatDateISO(weekStart);
            const [weekEvents, weekSlots] = await week_timetable(weekStartISO);

            events = weekEvents.map((e) => ({
                type: "event" as const,
                id: `${e.id.table}:${getKeyString(e.id.key)}`,
                title: e.name,
                description: e.description || undefined,
                displayStart: new Date(e.starts_at * 1000),
                displayEnd: new Date(e.ends_at * 1000),
                startedBefore: false,
                href: `/calendar/edit-event?id=${e.id.table}:${getKeyString(e.id.key)}`,
            }));

            slotsWithTasks = weekSlots.map((swt) => ({
                slot: swt.slot,
                tasks: swt.tasks,
            }));
        } catch (err) {
            console.error("Не удалось загрузить календарь недели:", err);
        } finally {
            loading = false;
        }
    }

    function getDayItems(dayIndex: number): CalendarDayItem[] {
        const dayStart = weekDays[dayIndex];
        if (!dayStart) return [];
        const nextDay = new Date(dayStart);
        nextDay.setDate(nextDay.getDate() + 1);
        const dayStartTs = dayStart.getTime();
        const nextDayTs = nextDay.getTime();

        const dayEvents: CalendarEvent[] = events
            .filter((e) => e.displayStart !== null && e.displayEnd !== null)
            .filter((e) => {
                const start = e.displayStart!.getTime();
                const end = e.displayEnd!.getTime();
                return start < nextDayTs && end > dayStartTs;
            })
            .map((e) => {
                const startedBefore = e.displayStart!.getTime() < dayStartTs;
                const endsAfter = e.displayEnd!.getTime() >= nextDayTs;

                return {
                    ...e,
                    displayStart: startedBefore ? null : e.displayStart,
                    displayEnd: endsAfter ? null : e.displayEnd,
                    startedBefore,
                };
            });

        const daySlots: CalendarSlot[] = slotsWithTasks
            .filter((swt) => {
                const slotStarts = swt.slot.starts_at * 1000;
                const slotEnds = swt.slot.ends_at * 1000;
                return slotStarts < nextDayTs && slotEnds > dayStartTs;
            })
            .map((swt) => {
                const slotStarts = swt.slot.starts_at * 1000;
                const slotEnds = swt.slot.ends_at * 1000;

                const startedBefore = slotStarts < dayStartTs;
                const endsAfter = slotEnds >= nextDayTs;

                const displayStart = startedBefore
                    ? dayStart
                    : new Date(slotStarts);
                const displayEnd = endsAfter
                    ? new Date(nextDayTs - 1)
                    : new Date(slotEnds);

                const filteredTasks = swt.tasks
                    .filter(([_, scheduledFor]) => {
                        return (
                            scheduledFor >= dayStartTs / 1000 &&
                            scheduledFor < nextDayTs / 1000
                        );
                    })
                    .sort((a, b) => {
                        const aStart = a[1];
                        const bStart = b[1];
                        const aEnd = aStart + (a[0].estimated_duration || 0);
                        const bEnd = bStart + (b[0].estimated_duration || 0);
                        const startDiff = aStart - bStart;
                        return startDiff !== 0 ? startDiff : aEnd - bEnd;
                    });

                return {
                    type: "slot" as const,
                    id: `${swt.slot.id?.table}:${getKeyString(swt.slot.id?.key)}`,
                    slot: swt.slot,
                    tasks: filteredTasks,
                    displayStart,
                    displayEnd,
                    startedBefore,
                    endsAfter,
                    href: `/calendar/edit-slot?id=${swt.slot.id?.table}:${getKeyString(swt.slot.id?.key)}`,
                };
            });

        return [...dayEvents, ...daySlots].sort((a, b) => {
            if (a.startedBefore && !b.startedBefore) return -1;
            if (!a.startedBefore && b.startedBefore) return 1;

            const aStart = a.displayStart?.getTime() ?? 0;
            const bStart = b.displayStart?.getTime() ?? 0;
            const startDiff = aStart - bStart;
            if (startDiff !== 0) return startDiff;

            const aEnd = a.displayEnd?.getTime() ?? 0;
            const bEnd = b.displayEnd?.getTime() ?? 0;
            return aEnd - bEnd;
        });
    }

    async function saveTaskState(
        taskId: string,
        completed: boolean,
    ): Promise<void> {
        const dayTasks = slotsWithTasks.flatMap((swt) =>
            swt.tasks.map(([task, _]) => ({
                task,
                id: `${task.id.table}:${getKeyString(task.id.key)}`,
            })),
        );
        const found = dayTasks.find((t) => t.id === taskId);
        if (!found) return;

        const taskObj: Task = {
            id: found.task.id,
            name: found.task.name,
            description: found.task.description,
            priority: found.task.priority,
            estimated_duration: found.task.estimated_duration,
            deadline: found.task.deadline,
            completed,
        };

        try {
            await update_task(taskObj);
            found.task.completed = completed;
        } catch (err) {
            console.error("Не удалось обновить задачу:", err);
        }
    }

    $effect(() => {
        const _ = weekStart;
        loadWeekData();
    });
</script>

<Page title="Календарь">
    {#snippet header()}
        <div class="flex items-center gap-4 h-full self-baseline">
            <a
                href="/calendar/create-event"
                class="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-full hover:bg-blue-700 transition-colors"
            >
                <CalendarPlus class="w-5 h-5" />
                <span>Создать событие</span>
            </a>
            <a
                href="/calendar/create-slot"
                class="flex items-center gap-2 px-4 py-2 bg-violet-600 text-white rounded-full hover:bg-violet-700 transition-colors"
            >
                <CopyPlus class="w-5 h-5" />
                <span>Создать слот</span>
            </a>
            <button
                onclick={goToPrevWeek}
                class="p-2 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors"
                aria-label="Previous week"
            >
                <ChevronLeft class="w-5 h-5 text-black dark:text-white" />
            </button>
            <button
                onclick={goToCurrentWeek}
                class="p-2 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors"
                aria-label="Current week"
            >
                <Circle class="w-5 h-5 text-black dark:text-white" />
            </button>
            <button
                onclick={goToNextWeek}
                class="p-2 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors"
                aria-label="Next week"
            >
                <ChevronRight class="w-5 h-5 text-black dark:text-white" />
            </button>
        </div>
    {/snippet}
    {#snippet body()}
        {#if loading}
            <div class="flex justify-center items-center h-32">
                <p class="text-gray-500">Загрузка...</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 lg:grid-cols-7 gap-4 h-full">
                {#each weekDays as day, index}
                    {@const dayItems = getDayItems(index)}
                    <div
                        class="flex flex-col gap-2 p-2 rounded-lg border-2 {isToday(
                            day,
                        )
                            ? 'border-slate-700 dark:border-slate-400'
                            : 'border-slate-400 dark:border-slate-600'} bg-slate-100 dark:bg-slate-800"
                    >
                        <div class="text-center">
                            <span
                                class="text-lg font-semibold text-black dark:text-white"
                                >{formatDay(day)}
                            </span>
                            <span
                                class="block text-sm text-gray-500 dark:text-gray-400"
                                >{formatMonth(day)}
                            </span>
                        </div>

                        <div class="flex flex-col gap-2">
                            {#each dayItems as item (item.id + "-" + index)}
                                {#if item.type === "event"}
                                    <EventCard
                                        title={item.title}
                                        description={item.description}
                                        startTime={item.displayStart}
                                        endTime={item.displayEnd}
                                        href={item.href}
                                    />
                                {:else}
                                    <Slot
                                        slot={item.slot}
                                        tasks={item.tasks}
                                        displayStart={item.displayStart}
                                        displayEnd={item.displayEnd}
                                        onTaskToggle={saveTaskState}
                                        href={item.href}
                                    />
                                {/if}
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    {/snippet}
</Page>
