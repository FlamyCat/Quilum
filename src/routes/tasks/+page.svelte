<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import TaskCard from "$lib/components/TaskCard.svelte";
    import { ListPlus, Trash2, CalendarClock } from "@lucide/svelte";
    import {
        get_all_task_lists,
        update_task_list,
        delete_task_list,
        update_task,
        run_scheduler,
        getKeyString,
        type TaskListWithTasks,
        type Task,
    } from "$lib/api";
    import { goto } from "$app/navigation";
    import { fade } from "svelte/transition";

    interface TaskListItem extends TaskListWithTasks {
        editingTitle: boolean;
        tempTitle: string;
    }

    let taskLists = $state<TaskListItem[]>([]);
    let loading = $state(true);
    let scheduling = $state(false);
    let notification = $state<{ message: string; type: "success" | "info" } | null>(null);
    let notificationVisible = $state(false);

    async function loadTaskLists() {
        loading = true;
        try {
            const lists = await get_all_task_lists();
            taskLists = lists.map((l) => ({
                ...l,
                editingTitle: false,
                tempTitle: l.list.title,
            }));
        } catch (err) {
            console.error("Не удалось загрузить списки задач:", err);
        } finally {
            loading = false;
        }
    }

    function startEditTitle(listId: string) {
        const list = taskLists.find(
            (l) =>
                `${l.list.id.table}:${getKeyString(l.list.id.key)}` === listId,
        );
        if (list) {
            list.editingTitle = true;
            list.tempTitle = list.list.title;
        }
    }

    async function saveTitle(listId: string) {
        const list = taskLists.find(
            (l) =>
                `${l.list.id.table}:${getKeyString(l.list.id.key)}` === listId,
        );
        if (list && list.tempTitle.trim()) {
            try {
                await update_task_list({
                    id: list.list.id,
                    title: list.tempTitle.trim(),
                });
                list.list.title = list.tempTitle.trim();
            } catch (err) {
                console.error("Не удалось обновить название списка:", err);
            }
            list.editingTitle = false;
        } else if (list) {
            list.editingTitle = false;
        }
    }

    function cancelEditTitle(listId: string) {
        const list = taskLists.find(
            (l) =>
                `${l.list.id.table}:${getKeyString(l.list.id.key)}` === listId,
        );
        if (list) {
            list.editingTitle = false;
            list.tempTitle = list.list.title;
        }
    }

    function handleTitleKeydown(event: KeyboardEvent, listId: string) {
        if (event.key === "Enter") {
            saveTitle(listId);
        } else if (event.key === "Escape") {
            cancelEditTitle(listId);
        }
    }

    async function handleDeleteList(listId: string) {
        if (
            !confirm(
                "Вы уверены, что хотите удалить этот список и все его задачи?",
            )
        ) {
            return;
        }

        const list = taskLists.find(
            (l) =>
                `${l.list.id.table}:${getKeyString(l.list.id.key)}` === listId,
        );
        if (!list) return;

        try {
            await delete_task_list(
                list.list.id.table,
                getKeyString(list.list.id.key),
            );
            taskLists = taskLists.filter(
                (l) =>
                    `${l.list.id.table}:${getKeyString(l.list.id.key)}` !==
                    listId,
            );
        } catch (err) {
            console.error("Не удалось удалить список:", err);
        }
    }

    async function handleTaskToggle(taskId: string, completed: boolean) {
        for (const list of taskLists) {
            const task = list.tasks.find(
                (t) => `${t.id.table}:${getKeyString(t.id.key)}` === taskId,
            );
            if (task) {
                try {
                    await update_task({
                        id: task.id,
                        name: task.name,
                        description: task.description,
                        priority: task.priority,
                        estimated_duration: task.estimated_duration,
                        deadline: task.deadline,
                        completed,
                    });
                    task.completed = completed;
                } catch (err) {
                    console.error("Не удалось обновить задачу:", err);
                }
                return;
            }
        }
    }

    async function handleRunScheduler() {
        scheduling = true;
        try {
            const result = await run_scheduler();
            if (result.scheduled > 0 && result.discarded.length === 0) {
                notification = { message: "Все задачи запланированы", type: "success" };
            } else if (result.scheduled === 0 && result.discarded.length === 0) {
                notification = { message: "Нет задач для планирования", type: "info" };
            } else if (result.discarded.length > 0) {
                const params = new URLSearchParams();
                params.set("discarded", result.discarded.join(","));
                goto(`/tasks/schedule?${params.toString()}`);
                return;
            }
            showNotification();
        } catch (err) {
            console.error("Ошибка планирования:", err);
            notification = { message: "Ошибка планирования", type: "info" };
            showNotification();
        } finally {
            scheduling = false;
        }
    }

    function showNotification() {
        notificationVisible = true;
        setTimeout(() => {
            notificationVisible = false;
        }, 3000);
    }

    $effect(() => {
        loadTaskLists();
    });
</script>

<Page title="Задачи">
    {#snippet header()}
        <div class="flex items-center gap-4 h-full self-baseline">
            {#if notification && notificationVisible}
                <div
                    role="alert"
                    class="px-4 py-2 rounded-lg text-white transition-opacity {notification.type === 'success' ? 'bg-teal-600' : 'bg-gray-600'}"
                    transition:fade={{ duration: 300 }}
                >
                    {notification.message}
                </div>
            {/if}
            <button
                onclick={handleRunScheduler}
                disabled={scheduling}
                class="flex items-center gap-2 px-4 py-2 bg-teal-600 text-white rounded-full hover:bg-teal-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
                {#if scheduling}
                    <span class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                {:else}
                    <CalendarClock class="w-5 h-5" />
                {/if}
                <span>Запланировать</span>
            </button>
            <a
                href="/tasks/create-list"
                class="flex items-center gap-2 px-4 py-2 bg-sky-600 text-white rounded-full hover:bg-sky-700 transition-colors"
            >
                <ListPlus class="w-5 h-5" />
                <span>Создать список</span>
            </a>
        </div>
    {/snippet}
    {#snippet body()}
        {#if loading}
            <div class="flex justify-center items-center h-32">
                <p class="text-gray-500">Загрузка...</p>
            </div>
        {:else}
            <div class="flex flex-wrap gap-4 h-full">
                {#each taskLists as listItem (listItem.list.id.table + ":" + getKeyString(listItem.list.id.key))}
                    {@const listId = `${listItem.list.id.table}:${getKeyString(listItem.list.id.key)}`}
                    {@const listTasks = listItem.tasks}
                    <div
                        class="w-100 flex flex-col h-[70vh] bg-slate-100 dark:bg-slate-800 rounded-lg overflow-hidden"
                    >
                        <div
                            class="flex items-center justify-between p-3 border-b border-slate-300 dark:border-slate-600"
                        >
                            {#if listItem.editingTitle}
                                <input
                                    type="text"
                                    bind:value={listItem.tempTitle}
                                    onblur={() => saveTitle(listId)}
                                    onkeydown={(e) =>
                                        handleTitleKeydown(e, listId)}
                                    class="flex-1 px-2 py-1 text-sm font-semibold bg-white dark:bg-slate-700 border border-slate-400 dark:border-slate-500 rounded text-black dark:text-white"
                                />
                            {:else}
                                <button
                                    onclick={() => startEditTitle(listId)}
                                    class="flex-1 text-left px-2 py-1 text-sm font-semibold text-black dark:text-white hover:bg-slate-200 dark:hover:bg-slate-700 rounded transition-colors truncate"
                                >
                                    {listItem.list.title}
                                </button>
                            {/if}
                            <button
                                onclick={() => handleDeleteList(listId)}
                                class="p-1 text-red-500 hover:bg-red-100 dark:hover:bg-red-900 rounded transition-colors ml-2"
                                aria-label="Удалить список"
                            >
                                <Trash2 class="w-4 h-4" />
                            </button>
                        </div>

                        <div
                            class="flex items-center p-2 border-b border-slate-300 dark:border-slate-600"
                        >
                            <a
                                href="/tasks/create?listId={listId}"
                                class="flex items-center gap-1 px-3 py-1.5 text-sm text-sky-600 dark:text-sky-400 hover:bg-slate-200 dark:hover:bg-slate-700 rounded transition-colors"
                            >
                                <ListPlus class="w-4 h-4" />
                                <span>Добавить задачу</span>
                            </a>
                        </div>

                        <div
                            class="flex-1 overflow-y-auto p-2 flex flex-col gap-2"
                        >
                            {#each listTasks as task (task.id.table + ":" + getKeyString(task.id.key))}
                                {@const taskId = `${task.id.table}:${getKeyString(task.id.key)}`}
                                <TaskCard
                                    title={task.name}
                                    description={task.description || undefined}
                                    startTime={null}
                                    endTime={null}
                                    completed={task.completed ?? false}
                                    onToggle={(completed) =>
                                        handleTaskToggle(taskId, completed)}
                                    href={"/tasks/edit?id=" + taskId}
                                    showTime={false}
                                />
                            {/each}
                            {#if listTasks.length === 0}
                                <p
                                    class="text-gray-500 dark:text-gray-400 text-sm text-center py-4"
                                >
                                    Нет задач
                                </p>
                            {/if}
                        </div>
                    </div>
                {/each}

                {#if taskLists.length === 0}
                    <div class="flex-1 flex items-center justify-center">
                        <p class="text-gray-500 dark:text-gray-400">
                            Нет списков задач. Создайте первый список!
                        </p>
                    </div>
                {/if}
            </div>
        {/if}
    {/snippet}
</Page>
