<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";

    let { children = () => null } = $props();

    $effect(() => {
        getCurrentWindow().setTheme(null);
    });
</script>

<div class="flex-1 p-4 rounded-2xl dark:bg-slate-900">
    {@render children()}
</div>
