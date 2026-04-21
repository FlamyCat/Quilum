<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import EventCard from "$lib/components/EventCard.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";
    import { ChevronLeft, ChevronRight, Circle, CalendarPlus } from "@lucide/svelte";
    import { eventsStore, getEventsForWeek, getTasksForWeek } from "$lib/stores/events";

    function getWeekStart(date: Date): Date {
        const d = new Date(date);
        const day = d.getDay();
        const diff = d.getDate() - day + (day === 0 ? -6 : 1);
        return new Date(d.setDate(diff));
    }

    const today = new Date();
    let weekStart = $state(getWeekStart(today));

    function getWeekOffset(): number {
        const currentWeekStart = getWeekStart(today);
        const diffTime = weekStart.getTime() - currentWeekStart.getTime();
        const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
        return Math.floor(diffDays / 7);
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
        return date.toLocaleString("en-US", { month: "short" }).toLowerCase();
    }

    function isToday(date: Date): boolean {
        return (
            date.getDate() === today.getDate() &&
            date.getMonth() === today.getMonth() &&
            date.getFullYear() === today.getFullYear()
        );
    }

    let mockEvents = $derived(getEventsForWeek(getWeekOffset()));

    let mockTasks = $state(getTasksForWeek(getWeekOffset()));

    function getEventsForDay(dayIndex: number) {
        return mockEvents.filter((e) => e.dayIndex === dayIndex);
    }

    function getTasksForDay(dayIndex: number) {
        return mockTasks.filter((t) => t.dayIndex === dayIndex);
    }

    async function saveTaskState(
        taskId: number,
        completed: boolean,
    ): Promise<void> {
        const task = mockTasks.find((t) => t.id === taskId);
        if (task) {
            task.completed = completed;
        }
    }
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
        <div class="grid grid-cols-1 lg:grid-cols-7 gap-4 h-full">
            {#each weekDays as day, index}
                {@const events = getEventsForDay(index)}
                {@const tasks = getTasksForDay(index)}
                <div
                    class="flex flex-col gap-2 p-2 rounded-lg border-2 {isToday(
                        day,
                    )
                        ? 'border-slate-600 dark:border-slate-400'
                        : 'border-slate-400 dark:border-slate-600'} bg-slate-300 dark:bg-slate-800"
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
                        {#each events as event}
                            <EventCard
                                title={event.title}
                                startTime={day}
                                endTime={day}
                            />
                        {/each}
                        {#each tasks as task}
                            <TaskCard
                                title={task.title}
                                startTime={day}
                                endTime={day}
                                completed={task.completed}
                                onToggle={(completed) =>
                                    saveTaskState(task.id, completed)}
                            />
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    {/snippet}
</Page>
