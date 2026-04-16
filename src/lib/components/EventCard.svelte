<script lang="ts">
    import BaseCard from "./BaseCard.svelte";

    type Props = {
        title: string;
        description?: string;
        startTime: Date;
        endTime: Date;
    };

    let { title, description, startTime, endTime }: Props = $props();
</script>

<BaseCard
    {title}
    {description}
    {startTime}
    {endTime}
    showCheckbox={false}
/>
