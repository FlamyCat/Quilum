<script lang="ts">
    let { children = () => null } = $props();
</script>

<div class="w-[min(10vw,90px)] p-2 flex items-center">
    {@render children()}
</div>
