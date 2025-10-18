<script lang="ts">
    import "../app.css";
    import Page from "./Body.svelte";
    import Navbar from "./Navbar.svelte";

    let { children } = $props();
</script>

<div class="flex gap-4 p-4 h-full">
    <Page>
        {@render children()}
    </Page>

    <Navbar />
</div>