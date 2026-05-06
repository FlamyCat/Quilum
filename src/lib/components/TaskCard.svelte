<script lang="ts">
    import BaseCard from "./BaseCard.svelte";

    type Props = {
        title: string;
        description?: string;
        startTime: Date | null;
        endTime: Date | null;
        completed?: boolean;
        onToggle?: (completed: boolean) => Promise<void>;
        showTime?: boolean;
    };

    let {
        title,
        description,
        startTime,
        endTime,
        completed = false,
        onToggle,
        showTime = false,
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
    {showTime}
    class="opacity-50 [&:not(.completed)]:opacity-100 [.completed_&]:opacity-50"
/>
