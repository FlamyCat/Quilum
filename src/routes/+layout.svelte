<script lang="ts">
    import "../app.css";
    import Page from "./Page.svelte";
    import Navbar from "./Navbar.svelte";

    let { children } = $props();
</script>

<div class="flex lg:flex-row flex-col h-full px-4 lg:pr-0 pt-4 lg:py-4 lg:pl-4">
    <Page>
        {@render children()}
    </Page>

    <Navbar />
</div>
