<script lang="ts">
    import Page from "$lib/components/Page.svelte";
    import { addEvent } from "$lib/stores/events";
    import { goto } from "$app/navigation";
    import Calendar from "$lib/components/ui/calendar/calendar.svelte";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import Input from "$lib/components/ui/input/input.svelte";
    import Label from "$lib/components/ui/label/label.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import { CalendarDate } from "@internationalized/date";
    import { cn } from "$lib/utils.js";

    import CalendarIcon from "@lucide/svelte/icons/calendar";

    let name = $state("");
    let description = $state("");

    const now = new Date();
    now.setMinutes(Math.ceil(now.getMinutes() / 15) * 15);

    let startDate = $state(
        new CalendarDate(now.getFullYear(), now.getMonth() + 1, now.getDate()),
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
        ),
    );
    let endTime = $state(
        `${String(endNow.getHours()).padStart(2, "0")}:${String(endNow.getMinutes()).padStart(2, "0")}`,
    );

    let startPopoverOpen = $state(false);
    let endPopoverOpen = $state(false);
    let error = $state("");

    function formatDate(date: CalendarDate): string {
        return date.toDate("UTC").toLocaleDateString("ru-RU");
    }

    function handleSubmit(event: Event) {
        event.preventDefault();
        error = "";

        if (!name.trim()) {
            error = "Название обязательно";
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

            <div class="flex flex-col gap-2">
                <Label for="startsAt" class="font-semibold">Начало *</Label>
                <div class="flex gap-2">
                    <Popover.Root bind:open={startPopoverOpen}>
                        <Popover.Trigger>
                            {#snippet child({ props })}
                                <Button
                                    variant="outline"
                                    class={cn(
                                        "w-35 justify-start text-left font-normal",
                                        !startDate && "text-muted-foreground",
                                    )}
                                    {...props}
                                >
                                    <CalendarIcon class="h-4 w-4" />
                                    {formatDate(startDate)}
                                </Button>
                            {/snippet}
                        </Popover.Trigger>
                        <Popover.Content class="w-auto p-0" align="start">
                            {#snippet child({ wrapperProps, props })}
                                <div {...wrapperProps}>
                                    <div {...props}>
                                        <Calendar
                                            type="single"
                                            bind:value={startDate}
                                            onValueChange={() => {
                                                startPopoverOpen = false;
                                            }}
                                        />
                                    </div>
                                </div>
                            {/snippet}
                        </Popover.Content>
                    </Popover.Root>
                    <Input
                        id="startTime"
                        type="time"
                        bind:value={startTime}
                        class="w-auto"
                    />
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <Label for="endsAt" class="font-semibold">Конец *</Label>
                <div class="flex gap-2">
                    <Popover.Root bind:open={endPopoverOpen}>
                        <Popover.Trigger>
                            {#snippet child({ props })}
                                <Button
                                    variant="outline"
                                    class={cn(
                                        "w-35 justify-start text-left font-normal",
                                        !endDate && "text-muted-foreground",
                                    )}
                                    {...props}
                                >
                                    <CalendarIcon class="h-4 w-4" />
                                    {formatDate(endDate)}
                                </Button>
                            {/snippet}
                        </Popover.Trigger>
                        <Popover.Content class="w-auto p-0" align="start">
                            {#snippet child({ wrapperProps, props })}
                                <div {...wrapperProps}>
                                    <div {...props}>
                                        <Calendar
                                            type="single"
                                            bind:value={endDate}
                                            onValueChange={() => {
                                                endPopoverOpen = false;
                                            }}
                                        />
                                    </div>
                                </div>
                            {/snippet}
                        </Popover.Content>
                    </Popover.Root>
                    <Input
                        id="endTime"
                        type="time"
                        bind:value={endTime}
                        class="w-auto"
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
