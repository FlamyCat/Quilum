<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import EventCard from "$lib/components/EventCard.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";

    function getWeekStart(date: Date): Date {
        const d = new Date(date);
        const day = d.getDay();
        const diff = d.getDate() - day + (day === 0 ? -6 : 1);
        return new Date(d.setDate(diff));
    }

    const today = new Date();
    let weekStart = $state(getWeekStart(today));

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

    const mockEvents = [
        { id: 1, title: "Team standup", dayIndex: 0, allDay: true },
        { id: 2, title: "Lunch break", dayIndex: 2, allDay: true },
        { id: 3, title: "Meeting", dayIndex: 4, allDay: true },
    ];

    let mockTasks = $state([
        { id: 1, title: "Buy groceries", dayIndex: 1, completed: false },
        { id: 2, title: "Read docs", dayIndex: 3, completed: false },
        { id: 3, title: "Send email", dayIndex: 5, completed: true },
    ]);

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
