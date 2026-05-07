<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { read_event, update_event, delete_event, getKeyString, type Event as CalendarEvent } from "$lib/api";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import DateTimePicker from "$lib/components/DateTimePicker.svelte";
    import { CalendarDate, type DateValue } from "@internationalized/date";
    import { onMount } from "svelte";

    let name = $state("");
    let description = $state("");
    let startDate = $state<DateValue | undefined>(undefined);
    let startTime = $state("00:00");
    let endDate = $state<DateValue | undefined>(undefined);
    let endTime = $state("01:00");
    let startPopoverOpen = $state(false);
    let endPopoverOpen = $state(false);
    let error = $state("");
    let loading = $state(true);
    let deleting = $state(false);
    let eventData: CalendarEvent | null = $state(null);

    onMount(() => {
        const urlParams = new URLSearchParams($page.url.search);
        const id = urlParams.get("id");

        if (!id) {
            error = "ID события не указан";
            loading = false;
            return;
        }

        const parts = id.split(":");
        if (parts.length !== 2) {
            error = "Неверный формат ID события";
            loading = false;
            return;
        }

        const idTable = parts[0];
        const idKey = parts[1];

        read_event(idTable, idKey)
            .then((event) => {
                eventData = event;
                name = event.name;
                description = event.description;

                const startDateTime = new Date(event.starts_at * 1000);
                startDate = new CalendarDate(
                    startDateTime.getFullYear(),
                    startDateTime.getMonth() + 1,
                    startDateTime.getDate(),
                ) as DateValue;
                startTime = `${String(startDateTime.getHours()).padStart(2, "0")}:${String(startDateTime.getMinutes()).padStart(2, "0")}`;

                const endDateTime = new Date(event.ends_at * 1000);
                endDate = new CalendarDate(
                    endDateTime.getFullYear(),
                    endDateTime.getMonth() + 1,
                    endDateTime.getDate(),
                ) as DateValue;
                endTime = `${String(endDateTime.getHours()).padStart(2, "0")}:${String(endDateTime.getMinutes()).padStart(2, "0")}`;

                loading = false;
            })
            .catch((err) => {
                error = "Ошибка при загрузке события: " + err;
                loading = false;
            });
    });

    function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        error = "";

        if (!name.trim()) {
            error = "Название обязательно";
            return;
        }

        if (!eventData) {
            error = "Событие не загружено";
            return;
        }

        const startDateValue = startDate;
        const endDateValue = endDate;
        if (!startDateValue || !endDateValue) {
            error = "Даты не указаны";
            return;
        }

        const startDateTime = new Date(
            startDateValue.year,
            startDateValue.month - 1,
            startDateValue.day,
            parseInt(startTime.split(":")[0]),
            parseInt(startTime.split(":")[1]),
        );
        const endDateTime = new Date(
            endDateValue.year,
            endDateValue.month - 1,
            endDateValue.day,
            parseInt(endTime.split(":")[0]),
            parseInt(endTime.split(":")[1]),
        );

        if (endDateTime <= startDateTime) {
            error = "Время окончания должно быть позже времени начала";
            return;
        }

        update_event({
            id: eventData.id,
            name: name.trim(),
            description: description.trim(),
            starts_at: Math.floor(startDateTime.getTime() / 1000),
            ends_at: Math.floor(endDateTime.getTime() / 1000),
        })
            .then(() => {
                window.history.back();
            })
            .catch((err) => {
                error = "Ошибка при обновлении события: " + err;
            });
    }

    function handleDelete() {
        if (!confirm("Вы уверены, что хотите удалить это событие?")) {
            return;
        }

        if (!eventData) {
            error = "Событие не загружено";
            return;
        }

        deleting = true;

        delete_event(eventData.id.table, getKeyString(eventData.id.key))
            .then(() => {
                window.history.back();
            })
            .catch((err) => {
                error = "Ошибка при удалении события: " + err;
                deleting = false;
            });
    }
</script>

<Page title="Редактирование события">
    {#snippet body()}
        {#if loading}
            <p class="text-slate-600 dark:text-slate-400">Загрузка...</p>
        {:else if error && !eventData}
            <p class="text-red-500">{error}</p>
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
                        placeholder="Введите название события"
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

                <DateTimePicker
                    label="Начало *"
                    id="startsAt"
                    bind:date={startDate}
                    bind:time={startTime}
                    bind:open={startPopoverOpen}
                />

                <DateTimePicker
                    label="Конец *"
                    id="endsAt"
                    bind:date={endDate}
                    bind:time={endTime}
                    bind:open={endPopoverOpen}
                />

                {#if error}
                    <p class="text-red-500">{error}</p>
                {/if}

                <div class="flex gap-4">
                    <button
                        type="submit"
                        class="px-6 py-3 bg-blue-600 text-white rounded-full hover:bg-blue-700 transition-colors font-semibold"
                    >
                        Сохранить
                    </button>

                    <button
                        type="button"
                        onclick={handleDelete}
                        disabled={deleting}
                        class="px-6 py-3 bg-red-600 text-white rounded-full hover:bg-red-700 transition-colors font-semibold disabled:opacity-50"
                    >
                        {deleting ? "Удаление..." : "Удалить"}
                    </button>
                </div>
            </form>
        {/if}
    {/snippet}
</Page>