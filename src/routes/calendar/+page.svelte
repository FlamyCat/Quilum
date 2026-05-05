<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import EventCard from "$lib/components/EventCard.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";
    import Slot from "$lib/components/Slot.svelte";
    import { ChevronLeft, ChevronRight, Circle, CalendarPlus, CopyPlus } from "@lucide/svelte";
    import { week_timetable, update_task, type Task, type Slot as ApiSlot, type SlotWithTasks } from "$lib/api";

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
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const day = String(date.getDate()).padStart(2, '0');
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

    function getKeyString(key: any): string {
        if (typeof key === 'string') return key;
        if (key && typeof key === 'object' && 'String' in key) return key.String;
        return String(key);
    }

    interface CalendarEvent {
        id: string;
        title: string;
        description?: string;
        startsAt: Date;
        endsAt: Date;
        displayStart: Date | null;
        displayEnd: Date | null;
        startedBefore: boolean;
    }

    interface CalendarSlot {
        id: string;
        slot: { starts_at: number; ends_at: number };
        tasks: [Task, number][];
        displayStart: Date;
        displayEnd: Date;
        startedBefore: boolean;
        endsAfter: boolean;
    }

    interface CalendarTask {
        id: string;
        title: string;
        description?: string;
        startsAt: Date;
        endsAt: Date;
        completed: boolean;
        taskIdTable: string;
        taskIdKey: string;
        priority: string;
        estimatedDuration: number;
        deadline: number;
    }

    let events = $state<CalendarEvent[]>([]);
    let slotsWithTasks = $state<SlotWithTasks[]>([]);
    let loading = $state(true);

    async function loadWeekData() {
        loading = true;
        try {
            const weekStartISO = formatDateISO(weekStart);
            const [weekEvents, weekSlots] = await week_timetable(weekStartISO);

            events = weekEvents.map(e => ({
                id: `${e.id.table}:${getKeyString(e.id.key)}`,
                title: e.name,
                description: e.description || undefined,
                startsAt: new Date(e.starts_at * 1000),
                endsAt: new Date(e.ends_at * 1000),
                displayStart: null,
                displayEnd: null,
                startedBefore: false,
            }));

            slotsWithTasks = weekSlots.map(swt => ({
                slot: swt.slot,
                tasks: swt.tasks,
            }));
        } catch (err) {
            console.error("Не удалось загрузить календарь недели:", err);
        } finally {
            loading = false;
        }
    }

    function getEventsForDay(dayIndex: number): CalendarEvent[] {
        const dayStart = weekDays[dayIndex];
        if (!dayStart) return [];
        const nextDay = new Date(dayStart);
        nextDay.setDate(nextDay.getDate() + 1);
        const dayStartTs = dayStart.getTime();
        const nextDayTs = nextDay.getTime();

        return events.filter(e => {
            const start = e.startsAt.getTime();
            const end = e.endsAt.getTime();
            // Event overlaps with this day if it starts before the day ends and ends after the day starts
            return start < nextDayTs && end > dayStartTs;
        }).map(e => {
            const startedBefore = e.startsAt.getTime() < dayStartTs;
            const endsAfter = e.endsAt.getTime() >= nextDayTs;
            
            return {
                ...e,
                displayStart: startedBefore ? null : e.startsAt,
                displayEnd: endsAfter ? null : e.endsAt,
                startedBefore,
            };
        }).sort((a, b) => {
            // Events that started before this day go to the top
            if (a.startedBefore && !b.startedBefore) return -1;
            if (!a.startedBefore && b.startedBefore) return 1;
            // Otherwise sort by display start time
            const aStart = a.displayStart?.getTime() ?? 0;
            const bStart = b.displayStart?.getTime() ?? 0;
            return aStart - bStart;
        });
    }

    function getSlotsForDay(dayIndex: number): CalendarSlot[] {
        const dayStart = weekDays[dayIndex];
        if (!dayStart) return [];
        const nextDay = new Date(dayStart);
        nextDay.setDate(nextDay.getDate() + 1);
        const dayStartTs = dayStart.getTime();
        const nextDayTs = nextDay.getTime();

        return slotsWithTasks.filter(swt => {
            const slotStarts = swt.slot.starts_at * 1000;
            const slotEnds = swt.slot.ends_at * 1000;
            return slotStarts < nextDayTs && slotEnds > dayStartTs;
        }).map(swt => {
            const slotStarts = swt.slot.starts_at * 1000;
            const slotEnds = swt.slot.ends_at * 1000;

            const startedBefore = slotStarts < dayStartTs;
            const endsAfter = slotEnds >= nextDayTs;

            const displayStart = startedBefore ? dayStart : new Date(slotStarts);
            const displayEnd = endsAfter ? new Date(nextDayTs - 1) : new Date(slotEnds);

            const filteredTasks = swt.tasks.filter(([_, scheduledFor]) => {
                return scheduledFor >= dayStartTs / 1000 && scheduledFor < nextDayTs / 1000;
            });

            return {
                id: `${swt.slot.id?.table}:${getKeyString(swt.slot.id?.key)}`,
                slot: swt.slot,
                tasks: filteredTasks,
                displayStart,
                displayEnd,
                startedBefore,
                endsAfter,
            };
        });
    }

    async function saveTaskState(taskId: string, completed: boolean): Promise<void> {
        const dayTasks = slotsWithTasks.flatMap(swt =>
            swt.tasks.map(([task, _]) => ({ task, id: `${task.id.table}:${task.id.key}` }))
        );
        const found = dayTasks.find(t => t.id === taskId);
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
                <ChevronLeft class="w-5 h-5" />
            </button>
            <button
                onclick={goToCurrentWeek}
                class="p-2 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors"
                aria-label="Current week"
            >
                <Circle class="w-5 h-5" />
            </button>
            <button
                onclick={goToNextWeek}
                class="p-2 rounded-lg hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors"
                aria-label="Next week"
            >
                <ChevronRight class="w-5 h-5" />
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
                    {@const dayEvents = getEventsForDay(index)}
                    {@const daySlots = getSlotsForDay(index)}
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
                             {#each dayEvents as event (event.id + '-' + index)}
                                <EventCard
                                    title={event.title}
                                    description={event.description}
                                    startTime={event.displayStart}
                                    endTime={event.displayEnd}
                                />
                            {/each}
                            {#each daySlots as calendarSlot (calendarSlot.id + '-' + index)}
                                <Slot
                                    slot={calendarSlot.slot}
                                    tasks={calendarSlot.tasks}
                                    displayStart={calendarSlot.displayStart}
                                    displayEnd={calendarSlot.displayEnd}
                                    onTaskToggle={saveTaskState}
                                />
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    {/snippet}
</Page>
