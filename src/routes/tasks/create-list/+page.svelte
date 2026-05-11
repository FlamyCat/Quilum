<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { create_task_list } from "$lib/api";
    import { goto } from "$app/navigation";

    let title = $state("");
    let error = $state("");

    function handleSubmit(event: Event) {
        event.preventDefault();
        error = "";

        if (!title.trim()) {
            error = "Название обязательно";
            return;
        }

        create_task_list(title.trim())
            .then(() => {
                goto("/tasks");
            })
            .catch((err) => {
                error = "Ошибка при создании списка: " + err;
            });
    }
</script>

<Page title="Создание списка">
    {#snippet body()}
        <form onsubmit={handleSubmit} class="flex flex-col gap-6 max-w-md">
            <div class="flex flex-col gap-2">
                <label
                    for="title"
                    class="font-semibold text-black dark:text-white"
                >
                    Название *
                </label>
                <input
                    id="title"
                    type="text"
                    bind:value={title}
                    class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    placeholder="Введите название списка"
                />
            </div>

            {#if error}
                <p class="text-red-500">{error}</p>
            {/if}

            <button
                type="submit"
                class="px-6 py-3 bg-sky-600 text-white rounded-full hover:bg-sky-700 transition-colors font-semibold self-start"
            >
                Создать список
            </button>
        </form>
    {/snippet}
</Page>