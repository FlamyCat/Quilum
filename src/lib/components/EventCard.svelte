<script lang="ts">
    import BaseCard from "./BaseCard.svelte";

type Props = {
    title: string;
    description?: string;
    startTime: Date | null;
    endTime: Date | null;
    href?: string;
};

    let { title, description, startTime, endTime, href }: Props = $props();
</script>

<BaseCard
    {title}
    {description}
    {startTime}
    {endTime}
    showCheckbox={false}
    {href}
    showTime={true}
/>
