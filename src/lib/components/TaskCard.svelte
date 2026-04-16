<script lang="ts">
    import BaseCard from "./BaseCard.svelte";

    type Props = {
        title: string;
        description?: string;
        startTime: Date;
        endTime: Date;
        completed?: boolean;
        onToggle?: (completed: boolean) => Promise<void>;
    };

    let {
        title,
        description,
        startTime,
        endTime,
        completed = false,
        onToggle,
    }: Props = $props();
</script>

<BaseCard
    {title}
    {description}
    {startTime}
    {endTime}
    showCheckbox={true}
    checked={completed}
    {onToggle}
    class="opacity-50 [&:not(.completed)]:opacity-100 [.completed_&]:opacity-50"
/>
