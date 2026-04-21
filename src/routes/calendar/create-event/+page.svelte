<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { addEvent } from "$lib/stores/events";
    import { goto } from "$app/navigation";


    let name = $state("");
    let description = $state("");

    const now = new Date();
    now.setMinutes(Math.ceil(now.getMinutes() / 15) * 15);

    function toLocalDateString(date: Date): string {
        const offset = date.getTimezoneOffset();
        const localDate = new Date(date.getTime() - offset * 60 * 1000);
        return localDate.toISOString().split("T")[0];
    }

    let startDate = $state(toLocalDateString(now));
    let startTime = $state(
        `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`
    );

    const endNow = new Date(now);
    endNow.setHours(endNow.getHours() + 1);
    let endDate = $state(toLocalDateString(endNow));
    let endTime = $state(
        `${String(endNow.getHours()).padStart(2, "0")}:${String(endNow.getMinutes()).padStart(2, "0")}`
    );

    let error = $state("");

    function handleSubmit(event: Event) {
        event.preventDefault();
        error = "";

        if (!name.trim()) {
            error = "Название обязательно";
            return;
        }

        const startDateTime = new Date(startDate + "T" + startTime);
        const endDateTime = new Date(endDate + "T" + endTime);

        if (endDateTime <= startDateTime) {
            error = "Время окончания должно быть позже времени начала";
            return;
        }

        const startDay = startDateTime.getDay();
        const dayIndex = startDay === 0 ? 6 : startDay - 1;

        addEvent({
            title: name.trim(),
            description: description.trim() || undefined,
            dayIndex,
            startsAt: startDateTime,
            endsAt: endDateTime,
        });

        goto("/calendar");
    }
</script>

<Page title="Создание события">
    {#snippet body()}
        <form onsubmit={handleSubmit} class="flex flex-col gap-6 max-w-md">
            <div class="flex flex-col gap-2">
                <label for="name" class="font-semibold text-black dark:text-white">
                    Название *
                </label>
                <input
                    id="name"
                    type="text"
                    bind:value={name}
                    class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    placeholder="Введите название события"
                />
            </div>

            <div class="flex flex-col gap-2">
                <label for="description" class="font-semibold text-black dark:text-white">
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
                <label for="startsAt" class="font-semibold text-black dark:text-white">
                    Начало *
                </label>
                <div class="flex gap-2">
                    <input
                        id="startDate"
                        type="date"
                        bind:value={startDate}
                        class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    />
                    <input
                        id="startTime"
                        type="time"
                        bind:value={startTime}
                        class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <label for="endsAt" class="font-semibold text-black dark:text-white">
                    Конец *
                </label>
                <div class="flex gap-2">
                    <input
                        id="endDate"
                        type="date"
                        bind:value={endDate}
                        class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    />
                    <input
                        id="endTime"
                        type="time"
                        bind:value={endTime}
                        class="px-4 py-2 rounded-lg border border-slate-400 dark:border-slate-600 bg-white dark:bg-slate-800 text-black dark:text-white"
                    />
                </div>
            </div>

            {#if error}
                <p class="text-red-500">{error}</p>
            {/if}

            <button
                type="submit"
                class="px-6 py-3 bg-blue-600 text-white rounded-full hover:bg-blue-700 transition-colors font-semibold self-start"
            >
                Создать событие
            </button>
        </form>
    {/snippet}
</Page>