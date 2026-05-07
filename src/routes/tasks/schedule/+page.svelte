<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { ArrowLeft, TriangleAlert } from "@lucide/svelte";
    import { get_all_task_lists, getKeyString, type Task } from "$lib/api";
    import { page as pageState } from "$app/state";
    import { onMount } from "svelte";

    let loading = $state(true);
    let discardedTasks = $state<Task[]>([]);

    onMount(async () => {
        const discardedParam = pageState.url.searchParams.get("discarded");
        if (!discardedParam) {
            loading = false;
            return;
        }

        const discardedIds = discardedParam.split(",");
        const discardedSet = new Set(discardedIds);

        try {
            const lists = await get_all_task_lists();

            const found: Task[] = [];
            for (const listItem of lists) {
                for (const task of listItem.tasks) {
                    const taskId = `${task.id.table}:${getKeyString(task.id.key)}`;
                    if (discardedSet.has(taskId)) {
                        found.push(task);
                    }
                }
            }
            discardedTasks = found;
        } catch (err) {
            console.error("Не удалось загрузить задачи:", err);
        } finally {
            loading = false;
        }
    });
</script>

<Page title="Результат планирования">
    {#snippet header()}
        <div class="flex items-center gap-4 h-full self-baseline">
            <a
                href="/tasks"
                class="flex items-center gap-2 px-4 py-2 bg-slate-600 text-white rounded-full hover:bg-slate-700 transition-colors"
            >
                <ArrowLeft class="w-5 h-5" />
                <span>К спискам</span>
            </a>
        </div>
    {/snippet}
    {#snippet body()}
        {#if loading}
            <div class="flex justify-center items-center h-32">
                <p class="text-gray-500">Загрузка...</p>
            </div>
        {:else}
            <div class="max-w-2xl mx-auto">
                <div class="flex items-center gap-3 mb-6 p-4 bg-amber-100 dark:bg-amber-900 rounded-lg">
                    <TriangleAlert class="w-6 h-6 text-amber-600 dark:text-amber-400 shrink-0" />
                    <h2 class="text-lg font-semibold text-amber-800 dark:text-amber-200">
                        Не все задачи удалось запланировать
                    </h2>
                </div>

                {#if discardedTasks.length > 0}
                    <div class="bg-slate-100 dark:bg-slate-800 rounded-lg overflow-hidden">
                        <div class="p-3 border-b border-slate-300 dark:border-slate-600">
                            <h3 class="font-semibold text-black dark:text-white">
                                Незапланированные задачи ({discardedTasks.length})
                            </h3>
                        </div>
                        <div class="divide-y divide-slate-200 dark:divide-slate-700">
                            {#each discardedTasks as task}
                                <div class="p-3">
                                    <p class="font-medium text-black dark:text-white">
                                        {task.name}
                                    </p>
                                    {#if task.description}
                                        <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                                            {task.description}
                                        </p>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    </div>
                {:else}
                    <p class="text-gray-500 text-center py-8">
                        Нет незапланированных задач
                    </p>
                {/if}
            </div>
        {/if}
    {/snippet}
</Page>