import { invoke } from '@tauri-apps/api/core';

export interface Event {
    id: { table: string; key: string };
    name: string;
    description: string;
    starts_at: number;
    ends_at: number;
}

export interface Slot {
    id: { table: string; key: string };
    starts_at: number;
    ends_at: number;
}

export interface Task {
    id: { table: string; key: string };
    name: string;
    description: string;
    priority: string;
    estimated_duration: number;
    deadline: number;
    completed?: boolean;
}

export interface SlotWithTasks {
    slot: Slot;
    tasks: [Task, number][];
}

export async function today_timetable(today: string): Promise<[Event[], [Task, number][]]> {
    return await invoke('today_timetable', { today });
}

export async function week_timetable(week_start: string): Promise<[Event[], SlotWithTasks[]]> {
    return await invoke('week_timetable', { weekStart: week_start });
}

export async function create_event(name: string, description: string, starts_at: number, ends_at: number): Promise<Event> {
    return await invoke('create_event', { name, description, startsAt: starts_at, endsAt: ends_at });
}

export async function read_event(id_table: string, id_key: string): Promise<Event> {
    return await invoke('read_event', { id_table, id_key });
}

export async function update_event(event: Event): Promise<void> {
    return await invoke('update_event', { event });
}

export async function delete_event(id_table: string, id_key: string): Promise<void> {
    return await invoke('delete_event', { id_table, id_key });
}

export async function create_slot(starts_at: number, ends_at: number): Promise<Slot> {
    return await invoke('create_slot', { startsAt: starts_at, endsAt: ends_at });
}

export async function read_slot(id_table: string, id_key: string): Promise<Slot> {
    return await invoke('read_slot', { id_table, id_key });
}

export async function update_slot(slot: Slot): Promise<void> {
    return await invoke('update_slot', { slot });
}

export async function delete_slot(id_table: string, id_key: string): Promise<void> {
    return await invoke('delete_slot', { id_table, id_key });
}

export async function create_task(name: string, description: string, priority: string, estimated_duration: number, deadline: number): Promise<Task> {
    return await invoke('create_task', { name, description, priority, estimatedDuration: estimated_duration, deadline });
}

export async function read_task(id_table: string, id_key: string): Promise<Task> {
    return await invoke('read_task', { id_table, id_key });
}

export async function update_task(task: Task): Promise<void> {
    return await invoke('update_task', { task });
}

export async function delete_task(id_table: string, id_key: string): Promise<void> {
    return await invoke('delete_task', { id_table, id_key });
}

export async function relate_task_to_slot(slot_id_table: string, slot_id_key: string, task_id_table: string, task_id_key: string, scheduled_for: number): Promise<void> {
    return await invoke('relate_task_to_slot', { slotIdTable: slot_id_table, slotIdKey: slot_id_key, taskIdTable: task_id_table, taskIdKey: task_id_key, scheduledFor: scheduled_for });
}
