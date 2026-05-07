<script lang="ts">
    import Page from "$lib/components/Page.svelte";
</script>

<Page title="Настройки">
    {#snippet body()}
        <div class="grid gap-4">
            <a href="/settings/app-blocking" class="block p-4 rounded-lg border hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors">
                <h2 class="text-lg font-semibold">Блокировка приложений</h2>
                <p class="text-sm text-gray-500 dark:text-gray-400">
                    Управление списком отвлекающих приложений
                </p>
            </a>
            <!-- Other settings cards can be added here later -->
        </div>
    {/snippet}
</Page>
