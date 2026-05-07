<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import PageTitle from "$lib/components/PageTitle.svelte";
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

    async function loadData() {
        try {
            loading = true;
            error = "";
            const [installed, blocked] = await Promise.all([
                invoke<AppInfo[]>("get_installed_apps"),
                invoke<AppInfo[]>("get_blocked_apps"),
            ]);
            installedApps = installed;
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
            const apps = blockedApps.map((app) => ({
                identifier: app.identifier,
                display_name: app.display_name,
            }));
            await invoke("update_blocked_apps", { apps });
            await loadData();
        } catch (e) {
            error = String(e);
            await loadData();
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
                <div
                    class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-lg"
                >
                    {error}
                </div>
            {/if}

            {#if loading}
                <div class="text-center py-8 text-gray-500">Загрузка...</div>
            {:else}
                <p class="mb-4 text-sm text-gray-500 dark:text-gray-400">
                    Выберите приложения, которые нужно блокировать во время
                    периода фокусировки
                </p>

                <div class="flex-1 overflow-y-auto space-y-2 mb-4">
                    {#each installedApps as app (app.identifier)}
                        <button
                            class="w-full text-left p-3 rounded-lg border transition-colors flex items-center justify-between {isBlocked(
                                app.identifier,
                            )
                                ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
                                : 'hover:bg-slate-100 dark:hover:bg-slate-800'}"
                            onclick={() =>
                                toggleApp(app.identifier, app.display_name)}
                        >
                            <span>{app.display_name}</span>
                            {#if isBlocked(app.identifier)}
                                <span class="text-red-500 text-sm"
                                    >Заблокировано</span
                                >
                            {/if}
                        </button>
                    {/each}
                </div>

                <button
                    class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50"
                    disabled={saving}
                    onclick={saveChanges}
                >
                    {saving ? "Сохранение..." : "Сохранить"}
                </button>
            {/if}
        </div>
    {/snippet}
</Page>
