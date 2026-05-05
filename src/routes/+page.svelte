<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import EventCard from "$lib/components/EventCard.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";
    import { today_timetable, update_task, type Task } from "$lib/api";

    function getTodayISO(): string {
        const now = new Date();
        const year = now.getFullYear();
        const month = String(now.getMonth() + 1).padStart(2, '0');
        const day = String(now.getDate()).padStart(2, '0');
        return `${year}-${month}-${day}`;
    }

    interface TimelineItem {
        id: string;
        title: string;
        description?: string;
        start: Date | null;  // null if event doesn't start today
        end: Date | null;    // null if event doesn't end today
        type: "event" | "task";
        completed?: boolean;
        taskIdTable?: string;
        taskIdKey?: string;
        priority?: string;
        estimatedDuration?: number;
        deadline?: number;
    }

    function getKeyString(key: any): string {
        if (typeof key === 'string') return key;
        if (key && typeof key === 'object' && 'String' in key) return key.String;
        return String(key);
    }

    let items: TimelineItem[] = $state([]);
    let loading = $state(true);

    async function loadTodayData() {
        loading = true;
        try {
            const today = getTodayISO();
            const [events, scheduledTasks] = await today_timetable(today);

            const eventItems: TimelineItem[] = events.map(e => {
                const eventStart = new Date(e.starts_at * 1000);
                const eventEnd = new Date(e.ends_at * 1000);
                const todayStart = new Date();
                todayStart.setHours(0, 0, 0, 0);
                const todayEnd = new Date(todayStart);
                todayEnd.setDate(todayEnd.getDate() + 1);

                const startsToday = eventStart >= todayStart && eventStart < todayEnd;
                const endsToday = eventEnd >= todayStart && eventEnd < todayEnd;

                return {
                    id: `${e.id.table}:${getKeyString(e.id.key)}`,
                    title: e.name,
                    description: e.description || undefined,
                    start: startsToday ? eventStart : null,
                    end: endsToday ? eventEnd : null,
                    type: "event" as const,
                };
            });

            const taskItems: TimelineItem[] = scheduledTasks.map(([task, scheduled_for]) => {
                return {
                    id: `${task.id.table}:${getKeyString(task.id.key)}`,
                    title: task.name,
                    description: task.description || undefined,
                    start: new Date(scheduled_for * 1000),
                    end: new Date((scheduled_for + task.estimated_duration) * 1000),
                    type: "task" as const,
                    completed: task.completed ?? false,
                    taskIdTable: task.id.table,
                    taskIdKey: task.id.key,
                    priority: task.priority,
                    estimatedDuration: task.estimated_duration,
                    deadline: task.deadline,
                };
            });

            items = [...eventItems, ...taskItems].sort((a, b) => {
                if (a.start === null && b.start === null) return 0;
                if (a.start === null) return 1;
                if (b.start === null) return -1;
                return a.start.getTime() - b.start.getTime();
            });
        } catch (err) {
            console.error("Не удалось загрузить план на сегодня:", err);
        } finally {
            loading = false;
        }
    }

    async function saveTaskState(taskId: string, completed: boolean): Promise<void> {
        const item = items.find(i => i.id === taskId && i.type === "task");
        if (!item || !item.taskIdTable || !item.taskIdKey) return;

        const task: Task = {
            id: { table: item.taskIdTable, key: item.taskIdKey },
            name: item.title,
            description: item.description || "",
            priority: item.priority || "medium",
            estimated_duration: item.estimatedDuration || 0,
            deadline: item.deadline || 0,
            completed,
        };

        try {
            await update_task(task);
            item.completed = completed;
        } catch (err) {
            console.error("Не удалось обновить задачу:", err);
        }
    }

    $effect(() => {
        loadTodayData();
    });
</script>

<Page title="План на сегодня">
    {#snippet body()}
        {#if loading}
            <div class="flex justify-center items-center h-32">
                <p class="text-gray-500">Загрузка...</p>
            </div>
        {:else}
            <div class="flex flex-col gap-4">
                {#each items as item (item.id + '-' + (item.start?.getTime() ?? 'null'))}
                    {#if item.type === "task"}
                        <TaskCard
                            title={item.title}
                            description={item.description}
                            startTime={item.start}
                            endTime={item.end}
                            completed={item.completed}
                            onToggle={(completed) => saveTaskState(item.id, completed)}
                        />
                    {:else}
                        <EventCard
                            title={item.title}
                            description={item.description}
                            startTime={item.start}
                            endTime={item.end}
                        />
                    {/if}
                {/each}
                {#if items.length === 0}
                    <p class="text-gray-500 text-center py-8">Нет событий и задач на сегодня.</p>
                {/if}
            </div>
        {/if}
    {/snippet}
</Page>
