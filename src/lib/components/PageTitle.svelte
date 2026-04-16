<script lang="ts">
    type Props = {
        title: string;
    };

    let { title }: Props = $props();
</script>

<h1 class="text-5xl font-bold text-left ml-6 my-4">{title}</h1>
