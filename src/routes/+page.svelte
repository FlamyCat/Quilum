<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import EventCard from "$lib/components/EventCard.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";

    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());

    const events = [
        {
            id: 1,
            title: "Team standup",
            description: "Daily sync with the team",
            start: new Date(today.getTime() + 9 * 60 * 60 * 1000),
            end: new Date(today.getTime() + 9 * 30 * 60 * 1000),
        },
        {
            id: 2,
            title: "Lunch break",
            description: "",
            start: new Date(today.getTime() + 12 * 60 * 60 * 1000),
            end: new Date(today.getTime() + 13 * 60 * 60 * 1000),
        },
    ];

    let tasks = $state([
        {
            id: 3,
            title: "Buy groceries",
            description: "Milk, eggs, bread",
            start: new Date(today.getTime() + 8 * 60 * 60 * 1000),
            end: new Date(today.getTime() + 8 * 30 * 60 * 1000),
            completed: false,
        },
        {
            id: 5,
            title: "Read documentation",
            description: "Review the new API docs before the meeting",
            start: new Date(today.getTime() + 10 * 60 * 60 * 1000),
            end: new Date(today.getTime() + 11 * 60 * 60 * 1000),
            completed: false,
        },
    ]);

    interface TimelineItem {
        id: number;
        title: string;
        description?: string;
        start: Date;
        end: Date;
        type: "event" | "task";
        completed?: boolean;
    }

    const items: TimelineItem[] = $derived([
        ...events.map((e) => ({ ...e, type: "event" as const })),
        ...tasks.map((t) => ({ ...t, type: "task" as const })),
    ].sort((a, b) => a.start.getTime() - b.start.getTime()));

    async function saveTaskState(taskId: number, completed: boolean): Promise<void> {
        const task = tasks.find((t) => t.id === taskId);
        if (task) {
            task.completed = completed;
        }
    }
</script>

<Page title="План на сегодня">
    {#snippet body()}
        <div class="flex flex-col gap-4">
            {#each items as item (item.id)}
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
        </div>
    {/snippet}
</Page>
