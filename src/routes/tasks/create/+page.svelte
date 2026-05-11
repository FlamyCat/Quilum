<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { create_task, relate_task_to_list, getKeyString } from "$lib/api";
    import { goto } from "$app/navigation";
    import DateTimePicker from "$lib/components/DateTimePicker.svelte";
    import { CalendarDate, type DateValue } from "@internationalized/date";
    import { page as pageState } from "$app/state";
    import * as Select from "$lib/components/ui/select/index.js";

    let name = $state("");
    let description = $state("");
    let priority = $state("Medium");
    let estimatedDurationMinutes = $state(30);
    let error = $state("");

    const now = new Date();
    now.setMinutes(Math.ceil(now.getMinutes() / 15) * 15);
    now.setHours(now.getHours() + 1);

    let deadlineDate = $state(
        new CalendarDate(
            now.getFullYear(),
            now.getMonth() + 1,
            now.getDate(),
        ) as DateValue,
    );
    let deadlineTime = $state(
        `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`,
    );

    let startPopoverOpen = $state(false);
    let errorMessage = $state("");

    function getListIdFromUrl(): { table: string; key: string } | null {
        const params = new URLSearchParams(pageState.url.search);
        const listId = params.get("listId");
        if (!listId) return null;

        const parts = listId.split(":");
        if (parts.length !== 2) return null;

        return { table: parts[0], key: parts[1] };
    }

    function handleSubmit(event: Event) {
        event.preventDefault();
        errorMessage = "";

        if (!name.trim()) {
            errorMessage = "Название обязательно";
            return;
        }

        if (estimatedDurationMinutes <= 0) {
            errorMessage = "Длительность должна быть больше 0";
            return;
        }

        const listId = getListIdFromUrl();
        if (!listId) {
            errorMessage = "Не указан список задач";
            return;
        }

        const deadlineDateTime = new Date(
            deadlineDate.year,
            deadlineDate.month - 1,
            deadlineDate.day,
            parseInt(deadlineTime.split(":")[0]),
            parseInt(deadlineTime.split(":")[1]),
        );

        const deadlineTimestamp = Math.floor(deadlineDateTime.getTime() / 1000);
        const estimatedDurationSeconds = estimatedDurationMinutes * 60;

        create_task(
            name.trim(),
            description.trim(),
            priority,
            estimatedDurationSeconds,
            deadlineTimestamp,
        )
            .then((task) => {
                return relate_task_to_list(
                    task.id.table,
                    getKeyString(task.id.key),
                    listId.table,
                    listId.key,
                );
            })
            .then(() => {
                goto("/tasks");
            })
            .catch((err) => {
                errorMessage = "Ошибка при создании задачи: " + err;
            });
    }
</script>

<Page title="Создание задачи">
    {#snippet body()}
        <form onsubmit={handleSubmit} class="flex flex-col gap-6 max-w-md">
            <div class="flex flex-col gap-2">
                <label
                    for="name"
                    class="font-semibold text-black dark:text-white"
                >
                    Название *
                </label>
                <input
                    id="name"
                    type="text"
                    bind:value={name}
                    class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    placeholder="Введите название задачи"
                />
            </div>

            <div class="flex flex-col gap-2">
                <label
                    for="description"
                    class="font-semibold text-black dark:text-white"
                >
                    Описание
                </label>
                <textarea
                    id="description"
                    bind:value={description}
                    rows="3"
                    class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white resize-none"
                    placeholder="Введите описание (необязательно)"
                ></textarea>
            </div>

            <div class="flex flex-col gap-2">
                <label
                    for="priority"
                    class="font-semibold text-black dark:text-white"
                >
                    Приоритет *
                </label>
                <Select.Root type="single" bind:value={priority as any}>
                    <Select.Trigger class="w-full px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white">
                        {#if priority === "Low"}Низкий{:else if priority === "Medium"}Средний{:else if priority === "High"}Высокий{:else}Выберите приоритет{/if}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="Low">Низкий</Select.Item>
                        <Select.Item value="Medium">Средний</Select.Item>
                        <Select.Item value="High">Высокий</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>

            <div class="flex flex-col gap-2">
                <label
                    for="duration"
                    class="font-semibold text-black dark:text-white"
                >
                    Длительность (минуты) *
                </label>
                <input
                    id="duration"
                    type="number"
                    bind:value={estimatedDurationMinutes}
                    min="1"
                    class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    placeholder="Введите длительность в минутах"
                />
            </div>

            <DateTimePicker
                label="Срок выполнения *"
                id="deadline"
                bind:date={deadlineDate}
                bind:time={deadlineTime}
                bind:open={startPopoverOpen}
            />

            {#if errorMessage}
                <p class="text-red-500">{errorMessage}</p>
            {/if}

            <button
                type="submit"
                class="px-6 py-3 bg-sky-600 text-white rounded-full hover:bg-sky-700 transition-colors font-semibold self-start"
            >
                Создать задачу
            </button>
        </form>
    {/snippet}
</Page>