<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    interface AppInfo {
        identifier: string;
        display_name: string;
    }

    let installedApps = $state<AppInfo[]>([]);
    let blockedApps = $state<AppInfo[]>([]);
    let loading = $state(true);
    let saving = $state(false);
    let error = $state("");
    let searchQuery = $state("");

    const filteredApps = $derived(
        installedApps.filter((app) =>
            app.display_name.toLowerCase().includes(searchQuery.toLowerCase()),
        ),
    );

    async function loadData() {
        try {
            loading = true;
            error = "";
            const [installed, blocked] = await Promise.all([
                invoke<AppInfo[]>("get_installed_apps"),
                invoke<AppInfo[]>("get_blocked_apps"),
            ]);
            installedApps = installed.toSorted((a, b) => {
                // Ignore upper and lowercase
                const nameA = a.display_name.toUpperCase();
                const nameB = b.display_name.toUpperCase();

                if (nameA < nameB) {
                    return -1;
                }

                if (nameA > nameB) {
                    return 1;
                }

                return 0;
            });
            blockedApps = blocked;
        } catch (e) {
            error = String(e);
        } finally {
            loading = false;
        }
    }

    function isBlocked(identifier: string): boolean {
        return blockedApps.some((app) => app.identifier === identifier);
    }

    async function toggleApp(identifier: string, displayName: string) {
        if (saving) return;

        const currentlyBlocked = isBlocked(identifier);

        if (currentlyBlocked) {
            blockedApps = blockedApps.filter(
                (app) => app.identifier !== identifier,
            );
        } else {
            blockedApps = [
                ...blockedApps,
                { identifier, display_name: displayName },
            ];
        }
    }

    async function saveChanges() {
        try {
            saving = true;
            error = "";
            const apps = blockedApps.map((app) => ( {
                identifier: app.identifier,
                display_name: app.display_name,
            } ));
            await invoke("update_blocked_apps", { apps });
        } catch (e) {
            error = String(e);
        } finally {
            saving = false;
        }
    }

    onMount(loadData);
</script>

<Page title="Блокировка приложений">
    {#snippet body()}
        <div class="flex flex-col h-full max-w-2xl p-4">
            {#if error}
                <div class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-lg">
                    {error}
                </div>
            {/if}

            {#if loading}
                <div class="text-center py-8 text-gray-500">Загрузка...</div>
            {:else}
                <p class="mb-4 text-sm text-gray-500 dark:text-gray-400">
                    Выберите приложения, которые нужно блокировать во время периода фокусировки </p>

                <input type="text"
                       class="w-full mb-4 p-4 rounded-lg bg-slate-100 dark:bg-slate-800 dark:border-slate-500 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                       placeholder="Поиск..."
                       bind:value={searchQuery} />

                <div class="flex-1 overflow-y-auto space-y-2 mb-4">
                    {#each filteredApps as app (app.identifier)}
                        <button class="w-full text-left p-3 rounded-lg border transition-colors flex items-center justify-between {isBlocked(
                                app.identifier,
                            )
                                ? 'bg-red-50 dark:bg-red-900/20 dark:hover:bg-red-800/30 border-red-200 dark:border-red-800'
                                : 'hover:bg-slate-100 dark:hover:bg-slate-700 dark:bg-slate-800 dark:border-slate-500'}"
                                onclick={() =>
                                toggleApp(app.identifier, app.display_name)}>
                            <span class="min-w-0 flex flex-col">
                                <span class="dark:text-white">{app.display_name}</span>
                                <span class="text-sm text-gray-500 truncate">{app.identifier}</span>
                            </span>
                            {#if isBlocked(app.identifier)}
                                <span class="text-red-500 text-sm shrink-0">Заблокировано</span>
                            {/if}
                        </button>
                    {/each}
                </div>

                <button class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
                        disabled={saving}
                        onclick={saveChanges}>
                    {saving ? "Сохранение..." : "Сохранить"}
                </button>
            {/if}
        </div>
    {/snippet}
</Page>
