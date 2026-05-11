<script lang="ts">
    import "../app.css";
    import Navbar from "./Navbar.svelte";

    let { children } = $props();
</script>

<div
    class="flex lg:flex-row flex-col h-screen overflow-hidden px-4 lg:pr-0 pt-4 lg:py-4 lg:pl-4 bg-slate-100 dark:bg-slate-950"
>
    <div class="flex-1 min-h-0">
        {@render children()}
    </div>

    <Navbar />
</div>
