<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { create_slot } from "$lib/api";
    import { goto } from "$app/navigation";
    import DateTimePicker from "$lib/components/DateTimePicker.svelte";
    import { CalendarDate, type DateValue } from "@internationalized/date";

    const now = new Date();
    now.setMinutes(Math.ceil(now.getMinutes() / 15) * 15);

    let startDate = $state(
        new CalendarDate(
            now.getFullYear(),
            now.getMonth() + 1,
            now.getDate(),
        ) as DateValue,
    );
    let startTime = $state(
        `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`,
    );

    const endNow = new Date(now);
    endNow.setHours(endNow.getHours() + 1);
    let endDate = $state(
        new CalendarDate(
            endNow.getFullYear(),
            endNow.getMonth() + 1,
            endNow.getDate(),
        ) as DateValue,
    );
    let endTime = $state(
        `${String(endNow.getHours()).padStart(2, "0")}:${String(endNow.getMinutes()).padStart(2, "0")}`,
    );

    let startPopoverOpen = $state(false);
    let endPopoverOpen = $state(false);
    let error = $state("");

    function handleSubmit(event: Event) {
        event.preventDefault();
        error = "";

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
            error = "Время окончания должно быть позже времени начала";
            return;
        }

        create_slot(
            Math.floor(startDateTime.getTime() / 1000),
            Math.floor(endDateTime.getTime() / 1000),
        ).then(() => {
            goto("/calendar");
        }).catch((err) => {
            error = "Ошибка при создании слота: " + err;
        });
    }
</script>

<Page title="Создание слота">
    {#snippet body()}
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

            {#if error}
                <p class="text-red-500">{error}</p>
            {/if}

            <button
                type="submit"
                class="px-6 py-3 bg-violet-600 text-white rounded-full hover:bg-violet-700 transition-colors font-semibold self-start"
            >
                Создать слот
            </button>
        </form>
    {/snippet}
</Page>