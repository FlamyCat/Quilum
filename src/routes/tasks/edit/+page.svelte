<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { read_task, update_task, delete_task, getKeyString } from "$lib/api";
    import { goto } from "$app/navigation";
    import DateTimePicker from "$lib/components/DateTimePicker.svelte";
    import { CalendarDate, type DateValue } from "@internationalized/date";
    import { onMount } from "svelte";
    import type { Task } from "$lib/api";

    let name = $state("");
    let description = $state("");
    let priority = $state("Medium");
    let estimatedDurationMinutes = $state(30);
    let error = $state("");

    let taskId = $state<{ table: string; key: string } | null>(null);
    let loadedTask = $state<Task | null>(null);

    let deadlineDate = $state(
        new CalendarDate(2024, 1, 1) as DateValue,
    );
    let deadlineTime = $state("12:00");

    let loading = $state(true);
    let errorMessage = $state("");
    let deleteLoading = $state(false);

    onMount(async () => {
        const params = new URLSearchParams(window.location.search);
        const idParam = params.get("id");
        if (!idParam) {
            errorMessage = "ID задачи не указан";
            loading = false;
            return;
        }

        const parts = idParam.split(":");
        if (parts.length !== 2) {
            errorMessage = "Неверный формат ID задачи";
            loading = false;
            return;
        }

        taskId = { table: parts[0], key: parts[1] };

        try {
            const task: Task = await read_task(taskId.table, taskId.key);
            loadedTask = task;
            name = task.name;
            description = task.description || "";
            priority = task.priority || "Medium";
            estimatedDurationMinutes = Math.round(task.estimated_duration / 60);

            const deadlineDateObj = new Date(task.deadline * 1000);
            deadlineDate = new CalendarDate(
                deadlineDateObj.getFullYear(),
                deadlineDateObj.getMonth() + 1,
                deadlineDateObj.getDate(),
            ) as DateValue;
            deadlineTime = `${String(deadlineDateObj.getHours()).padStart(2, "0")}:${String(deadlineDateObj.getMinutes()).padStart(2, "0")}`;

            loading = false;
        } catch (err) {
            errorMessage = "Ошибка при загрузке задачи: " + err;
            loading = false;
        }
    });

    function handleSubmit(event: Event) {
        event.preventDefault();
        errorMessage = "";
        if (!taskId) {
            errorMessage = "ID задачи не определен";
            return;
        }

        if (!name.trim()) {
            errorMessage = "Название обязательно";
            return;
        }

        if (estimatedDurationMinutes <= 0) {
            errorMessage = "Длительность должна быть больше 0";
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

        const updatedTask: Task = {
            id: loadedTask!.id,
            name: name.trim(),
            description: description.trim(),
            priority,
            estimated_duration: estimatedDurationSeconds,
            deadline: deadlineTimestamp,
            completed: loadedTask!.completed,
        };

        update_task(updatedTask)
            .then(() => {
                window.history.back();
            })
            .catch((err) => {
                errorMessage = "Ошибка при сохранении задачи: " + err;
            });
    }

    function handleDelete() {
        if (!taskId) return;

        const confirmed = confirm("Вы уверены, что хотите удалить эту задачу?");
        if (!confirmed) return;

        deleteLoading = true;

        delete_task(taskId.table, taskId.key)
            .then(() => {
                window.history.back();
            })
            .catch((err) => {
                errorMessage = "Ошибка при удалении задачи: " + err;
                deleteLoading = false;
            });
    }
</script>

<Page title="Редактирование задачи">
    {#snippet body()}
        {#if loading}
            <div class="text-gray-500">Загрузка...</div>
        {:else if errorMessage && !taskId}
            <div class="text-red-500">{errorMessage}</div>
        {:else}
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
                    <select
                        id="priority"
                        bind:value={priority}
                        class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    >
                        <option value="Low">Низкий</option>
                        <option value="Medium">Средний</option>
                        <option value="High">Высокий</option>
                    </select>
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
                />

                {#if errorMessage}
                    <p class="text-red-500">{errorMessage}</p>
                {/if}

                <div class="flex gap-4">
                    <button
                        type="submit"
                        class="px-6 py-3 bg-sky-600 text-white rounded-full hover:bg-sky-700 transition-colors font-semibold self-start"
                    >
                        Сохранить
                    </button>
                    <button
                        type="button"
                        onclick={handleDelete}
                        disabled={deleteLoading}
                        class="px-6 py-3 bg-red-600 text-white rounded-full hover:bg-red-700 transition-colors font-semibold self-start disabled:opacity-50"
                    >
                        {deleteLoading ? "Удаление..." : "Удалить"}
                    </button>
                </div>
            </form>
        {/if}
    {/snippet}
</Page>