<script lang="ts">
    import type { Task } from "$lib/api";
    import { getKeyString } from "$lib/api";
    import TaskCard from "./TaskCard.svelte";

    type Props = {
        slot: { starts_at: number; ends_at: number };
        tasks: [Task, number][];
        onTaskToggle?: (taskId: string, completed: boolean) => Promise<void>;
        displayStart: Date;
        displayEnd: Date;
        startedBefore?: boolean;
        endsAfter?: boolean;
        href?: string;
    };

    let { slot, tasks, onTaskToggle, displayStart, displayEnd, href }: Props = $props();

    let isMultiDay = $derived(
        displayStart.getDate() !== displayEnd.getDate() ||
        displayStart.getMonth() !== displayEnd.getMonth() ||
        displayStart.getFullYear() !== displayEnd.getFullYear()
    );

    function formatDateTime(date: Date): string {
        const dayMonth = date.toLocaleString("ru-RU", { day: "numeric", month: "short" });
        const time = date.toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" });
        return `${dayMonth} ${time}`;
    }

    function formatTime(date: Date): string {
        return date.toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" });
    }

    function getTaskId(task: Task): string {
        return `${task.id.table}:${getKeyString(task.id.key)}`;
    }
</script>

<div class="rounded-lg bg-violet-400 dark:bg-violet-900 border-violet-600 dark:border-violet-500 p-2 flex flex-col gap-2 border-2">
    {#if href}
        <a href={href} class="text-center font-semibold text-white dark:text-violet-100 cursor-pointer">Слот</a>
    {:else}
        <div class="text-center font-semibold text-white dark:text-violet-100">Слот</div>
    {/if}
    {#if !isMultiDay}
        <div class="text-center text-white dark:text-violet-200">{formatTime(displayStart)} - {formatTime(displayEnd)}</div>
    {:else}
        <div class="flex flex-col items-center">
            <span>{formatDateTime(displayStart)}</span>
            <span class="text-white dark:text-violet-200">-</span>
            <span>{formatDateTime(displayEnd)}</span>
        </div>
    {/if}
    {#each tasks as [task, scheduled_for]}
        <TaskCard
            title={task.name}
            description={task.description}
            startTime={new Date(scheduled_for * 1000)}
            endTime={new Date((scheduled_for + task.estimated_duration) * 1000)}
            completed={task.completed ?? false}
            onToggle={onTaskToggle ? async (completed) => {
                await onTaskToggle(getTaskId(task), completed);
            } : undefined}
            showTime={true}
            href={"/tasks/edit?id=" + task.id.table + ":" + getKeyString(task.id.key)}
        />
    {/each}
</div>