<script lang="ts">
	import { Select as SelectPrimitive } from "bits-ui";

	type SelectValue = string | string[] | undefined;

	let {
		open = $bindable(false),
		value = $bindable<SelectValue>(),
		...restProps
	}: SelectPrimitive.RootProps = $props();
</script>

<SelectPrimitive.Root bind:open bind:value={value as any} {...restProps} />
