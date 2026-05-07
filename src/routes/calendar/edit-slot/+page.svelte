<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { read_slot, update_slot, delete_slot, getKeyString } from "$lib/api";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import DateTimePicker from "$lib/components/DateTimePicker.svelte";
    import { CalendarDate, type DateValue } from "@internationalized/date";
    import { onMount } from "svelte";

    let loading = $state(true);
    let error = $state("");
    let slot = $state<{ id: { table: string; key: string }; starts_at: number; ends_at: number } | null>(null);

    let startDate = $state<DateValue | undefined>(undefined);
    let startTime = $state("");
    let endDate = $state<DateValue | undefined>(undefined);
    let endTime = $state("");

    let startPopoverOpen = $state(false);
    let endPopoverOpen = $state(false);
    let submitError = $state("");

    onMount(() => {
        const idParam = new URLSearchParams($page.url.search).get("id");
        if (!idParam) {
            error = "ID слота не найден";
            loading = false;
            return;
        }

        const parts = idParam.split(":");
        if (parts.length !== 2) {
            error = "Неверный формат ID";
            loading = false;
            return;
        }

        const id_table = parts[0];
        const id_key = parts[1];

        read_slot(id_table, id_key)
            .then((data) => {
                slot = data;
                const startDateObj = new Date(data.starts_at * 1000);
                const endDateObj = new Date(data.ends_at * 1000);

                startDate = new CalendarDate(
                    startDateObj.getFullYear(),
                    startDateObj.getMonth() + 1,
                    startDateObj.getDate(),
                ) as DateValue;
                startTime = `${String(startDateObj.getHours()).padStart(2, "0")}:${String(startDateObj.getMinutes()).padStart(2, "0")}`;

                endDate = new CalendarDate(
                    endDateObj.getFullYear(),
                    endDateObj.getMonth() + 1,
                    endDateObj.getDate(),
                ) as DateValue;
                endTime = `${String(endDateObj.getHours()).padStart(2, "0")}:${String(endDateObj.getMinutes()).padStart(2, "0")}`;

                loading = false;
            })
            .catch((err) => {
                error = "Ошибка при загрузке слота: " + err;
                loading = false;
            });
    });

    function handleSubmit(event: Event) {
        event.preventDefault();
        submitError = "";

        if (!slot || !startDate || !endDate) {
            submitError = "Данные слота не загружены";
            return;
        }

        const startDateTime = new Date(
            startDate.year,
            startDate.month - 1,
            startDate.day,
            parseInt(startTime.split(":")[0]),
            parseInt(startTime.split(":")[1]),
        );
        const endDateTime = new Date(
            endDate.year,
            endDate.month - 1,
            endDate.day,
            parseInt(endTime.split(":")[0]),
            parseInt(endTime.split(":")[1]),
        );

        if (endDateTime <= startDateTime) {
            submitError = "Время окончания должно быть позже времени начала";
            return;
        }

        update_slot({
            id: slot.id,
            starts_at: Math.floor(startDateTime.getTime() / 1000),
            ends_at: Math.floor(endDateTime.getTime() / 1000),
        })
            .then(() => {
                window.history.back();
            })
            .catch((err) => {
                submitError = "Ошибка при сохранении: " + err;
            });
    }

    function handleDelete() {
        if (!slot) return;

        const confirmed = confirm("Вы уверены, что хотите удалить этот слот?");
        if (!confirmed) return;

        delete_slot(slot.id.table, getKeyString(slot.id.key))
            .then(() => {
                window.history.back();
            })
            .catch((err) => {
                submitError = "Ошибка при удалении: " + err;
            });
    }
</script>

<Page title="Редактирование слота">
    {#snippet body()}
        {#if loading}
            <p>Загрузка...</p>
        {:else if error}
            <p class="text-red-500">{error}</p>
        {:else if slot}
            <form onsubmit={handleSubmit} class="flex flex-col gap-6 max-w-md">
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

                {#if submitError}
                    <p class="text-red-500">{submitError}</p>
                {/if}

                <div class="flex gap-4">
                    <button
                        type="submit"
                        class="px-6 py-3 bg-violet-600 text-white rounded-full hover:bg-violet-700 transition-colors font-semibold"
                    >
                        Сохранить
                    </button>

                    <button
                        type="button"
                        onclick={handleDelete}
                        class="px-6 py-3 bg-red-600 text-white rounded-full hover:bg-red-700 transition-colors font-semibold"
                    >
                        Удалить
                    </button>
                </div>
            </form>
        {/if}
    {/snippet}
</Page>