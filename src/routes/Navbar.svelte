<script lang="ts">
    import { page } from "$app/state";
    import { CalendarCheck, Calendar1, ListTodo, Settings } from "@lucide/svelte";
    import { DarkMode } from "flowbite-svelte";

    const navItems = [
        { href: "/", icon: CalendarCheck, label: "Сегодня" },
        { href: "/calendar", icon: Calendar1, label: "Календарь" },
        { href: "/tasks", icon: ListTodo, label: "Задачи" },
        { href: "/settings", icon: Settings, label: "Настройки" },
    ];
</script>

<nav class="lg:w-auto sm:w-full flex sm:flex-row lg:flex-col gap-4 p-4 items-center justify-center sm:static bg-transparent">
    {#each navItems as item}
        {@const isActive = page.url.pathname === item.href}
        <a
            href={item.href}
            aria-label={item.label}
            class="flex flex-col items-center gap-1 transition-colors duration-200 {isActive
                ? 'text-black dark:text-white'
                : 'text-gray-500 dark:text-gray-400'}"
        >
            <item.icon size={24} strokeWidth={2} />
            <span class="text-sm font-medium">{item.label}</span>
        </a>
    {/each}
    <DarkMode />
</nav>
