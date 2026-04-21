export interface CalendarEvent {
    id: number;
    title: string;
    description?: string;
    dayIndex: number;
    startsAt: Date;
    endsAt: Date;
}

export interface CalendarTask {
    id: number;
    title: string;
    dayIndex: number;
    completed: boolean;
}

const mockEventsByWeek: Record<number, CalendarEvent[]> = {
    [-1]: [
        { id: 101, title: "Sprint planning", dayIndex: 1, startsAt: new Date(), endsAt: new Date() },
        { id: 102, title: "1:1 with manager", dayIndex: 3, startsAt: new Date(), endsAt: new Date() },
    ],
    [0]: [
        { id: 1, title: "Team standup", dayIndex: 0, startsAt: new Date(), endsAt: new Date() },
        { id: 2, title: "Lunch break", dayIndex: 2, startsAt: new Date(), endsAt: new Date() },
        { id: 3, title: "Meeting", dayIndex: 4, startsAt: new Date(), endsAt: new Date() },
    ],
    [1]: [
        { id: 201, title: "Code review", dayIndex: 0, startsAt: new Date(), endsAt: new Date() },
        { id: 202, title: "Team lunch", dayIndex: 2, startsAt: new Date(), endsAt: new Date() },
        { id: 203, title: "Project demo", dayIndex: 4, startsAt: new Date(), endsAt: new Date() },
    ],
};

const mockTasksByWeek: Record<number, CalendarTask[]> = {
    [-1]: [
        { id: 101, title: "Write report", dayIndex: 0, completed: true },
        { id: 102, title: "Call client", dayIndex: 2, completed: false },
        { id: 103, title: "Prepare slides", dayIndex: 4, completed: true },
    ],
    [0]: [
        { id: 1, title: "Buy groceries", dayIndex: 1, completed: false },
        { id: 2, title: "Read docs", dayIndex: 3, completed: false },
        { id: 3, title: "Send email", dayIndex: 5, completed: true },
    ],
    [1]: [
        { id: 201, title: "Fix bugs", dayIndex: 1, completed: false },
        { id: 202, title: "Update documentation", dayIndex: 3, completed: false },
        { id: 203, title: "Deploy to staging", dayIndex: 5, completed: false },
    ],
};

export const eventsStore = {
    events: mockEventsByWeek,
    tasks: mockTasksByWeek,
    nextEventId: 300,
    nextTaskId: 300,
};

export function getEventsForWeek(weekOffset: number): CalendarEvent[] {
    return eventsStore.events[weekOffset] ?? [...(eventsStore.events[0] ?? [])];
}

export function getTasksForWeek(weekOffset: number): CalendarTask[] {
    return eventsStore.tasks[weekOffset] ?? [...(eventsStore.tasks[0] ?? [])].map(t => ({ ...t }));
}

export function addEvent(event: Omit<CalendarEvent, "id">): void {
    const newEvent: CalendarEvent = {
        ...event,
        id: eventsStore.nextEventId++,
    };
    const weekOffset = 0;
    if (!eventsStore.events[weekOffset]) {
        eventsStore.events[weekOffset] = [];
    }
    eventsStore.events[weekOffset].push(newEvent);
}