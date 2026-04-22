<script lang="ts">
    import Calendar from "$lib/components/ui/calendar/calendar.svelte";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import Input from "$lib/components/ui/input/input.svelte";
    import Label from "$lib/components/ui/label/label.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import { type DateValue } from "@internationalized/date";
    import { cn } from "$lib/utils.js";
    import CalendarIcon from "@lucide/svelte/icons/calendar";

    let {
        label,
        id,
        date = $bindable<DateValue>(),
        time = $bindable<string>(""),
        open = $bindable(false),
    }: {
        label: string;
        id: string;
        date?: DateValue;
        time?: string;
        open?: boolean;
    } = $props();

    export function formatDateValue(d: DateValue): string {
        return d.toDate("UTC").toLocaleDateString("ru-RU");
    }

    function handleDateSelect(d: DateValue | undefined) {
        if (d) {
            date = d;
            open = false;
        }
    }
</script>

<div class="flex flex-col gap-2">
    <Label for={id} class="font-semibold">{label}</Label>
    <div class="flex gap-2">
        <Popover.Root bind:open>
            <Popover.Trigger>
                {#snippet child({ props: triggerProps })}
                    <Button
                        variant="outline"
                        class={cn(
                            "w-35 justify-start text-left font-normal",
                            !date && "text-muted-foreground",
                        )}
                        {...triggerProps}
                    >
                        <CalendarIcon class="h-4 w-4" />
                        {date ? formatDateValue(date) : ""}
                    </Button>
                {/snippet}
            </Popover.Trigger>
            <Popover.Content class="w-auto p-0" align="start">
                {#snippet child({ wrapperProps, props: contentProps })}
                    <div {...wrapperProps}>
                        <div {...contentProps}>
                            <Calendar
                                type="single"
                                bind:value={date}
                                onValueChange={handleDateSelect}
                            />
                        </div>
                    </div>
                {/snippet}
            </Popover.Content>
        </Popover.Root>
        <Input
            id={id.replace("At", "Time")}
            type="time"
            bind:value={time}
            class="w-auto"
        />
    </div>
</div>
