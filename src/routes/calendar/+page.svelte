<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import EventCard from "$lib/components/EventCard.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";
    import { ChevronLeft, ChevronRight, Circle } from "@lucide/svelte";

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

    type Event = { id: number; title: string; dayIndex: number };
    type Task = { id: number; title: string; dayIndex: number; completed: boolean };

    const mockEventsByWeek: Record<number, Event[]> = {
        [-1]: [
            { id: 101, title: "Sprint planning", dayIndex: 1 },
            { id: 102, title: "1:1 with manager", dayIndex: 3 },
        ],
        [0]: [
            { id: 1, title: "Team standup", dayIndex: 0 },
            { id: 2, title: "Lunch break", dayIndex: 2 },
            { id: 3, title: "Meeting", dayIndex: 4 },
        ],
        [1]: [
            { id: 201, title: "Code review", dayIndex: 0 },
            { id: 202, title: "Team lunch", dayIndex: 2 },
            { id: 203, title: "Project demo", dayIndex: 4 },
        ],
    };

    const mockTasksByWeek: Record<number, Task[]> = {
        [-1]: [
            { id: 101, title: "Write report", dayIndex: 0, completed: true },
            { id: 102, title: "Call client", dayIndex: 2, completed: false },
            { id: 103, title: "Prepare slides", dayIndex: 4, completed: true },
        ],
        [0]: [
            { id: 1, title: "Buy groceries", dayIndex: 1, completed: false },
            { id: 2, title: "Read docs", dayIndex: 3, completed: false },
            { id: 3, title: "Send email", dayIndex: 5, completed: true },
        ],
        [1]: [
            { id: 201, title: "Fix bugs", dayIndex: 1, completed: false },
            { id: 202, title: "Update documentation", dayIndex: 3, completed: false },
            { id: 203, title: "Deploy to staging", dayIndex: 5, completed: false },
        ],
    };

    let mockEvents = $derived(
        mockEventsByWeek[getWeekOffset()] ?? mockEventsByWeek[0]
    );

    let mockTasks = $state<Task[]>(
        mockTasksByWeek[getWeekOffset()] ?? [...mockTasksByWeek[0]].map(t => ({ ...t }))
    );

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
        <div class="flex items-center gap-2 h-full self-baseline">
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
