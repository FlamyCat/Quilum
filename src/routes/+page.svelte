<script lang="ts">
    import Page from "$lib/components/Page.svelte";
</script>

<Page title="План на сегодня">
    {#snippet body()}
        <p>Здесь будет план на сегодня</p>
    {/snippet}
</Page>
