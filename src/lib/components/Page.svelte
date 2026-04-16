<script lang="ts">
    import type { Snippet } from "svelte";
    import PageTitle from "./PageTitle.svelte";

    type Props = {
        title: string;
        header?: Snippet;
        body: Snippet;
    };

    let { title, header, body }: Props = $props();
</script>

<div class="flex flex-col gap-4 size-full">
    <header>
        <PageTitle {title} />
        {#if header}
            {@render header()}
        {/if}
    </header>

    <div class="flex-1 p-8 rounded-2xl bg-slate-50 dark:bg-slate-900 size-full">
        {@render body()}
    </div>
</div>
