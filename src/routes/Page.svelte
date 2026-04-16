<script lang="ts">
    let { children = () => null } = $props();
</script>

<div class="flex-1 p-4 rounded-2xl dark:bg-slate-900">
    {@render children()}
</div>
