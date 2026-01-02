import { create } from "zustand";

type ExecutingTasksState = {
	/** Set of task IDs that are currently executing (have running sessions) */
	executingTaskIds: Set<string>;
	/** Mark a task as executing */
	startExecuting: (taskId: string) => void;
	/** Mark a task as no longer executing */
	stopExecuting: (taskId: string) => void;
	/** Check if a task is executing */
	isExecuting: (taskId: string) => boolean;
};

export const useExecutingTasksStore = create<ExecutingTasksState>()((set, get) => ({
	executingTaskIds: new Set<string>(),
	startExecuting: (taskId) =>
		set((state) => ({
			executingTaskIds: new Set([...state.executingTaskIds, taskId]),
		})),
	stopExecuting: (taskId) =>
		set((state) => {
			const newSet = new Set(state.executingTaskIds);
			newSet.delete(taskId);
			return { executingTaskIds: newSet };
		}),
	isExecuting: (taskId) => get().executingTaskIds.has(taskId),
}));

export const useIsTaskExecuting = (taskId: string) =>
	useExecutingTasksStore((s) => s.executingTaskIds.has(taskId));
