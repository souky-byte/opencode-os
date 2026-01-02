import NiceModal, { useModal } from "@ebay/nice-modal-react";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { getListTasksQueryKey, useCreateTask } from "@/api/generated/tasks/tasks";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { defineModal } from "@/lib/modals";

interface CreateTaskDialogProps {
	initialTitle?: string;
}

const CreateTaskDialogComponent = NiceModal.create<CreateTaskDialogProps>(({ initialTitle }) => {
	const modal = useModal();
	const queryClient = useQueryClient();
	const [title, setTitle] = useState(initialTitle ?? "");
	const [description, setDescription] = useState("");

	const createTask = useCreateTask({
		mutation: {
			onSuccess: () => {
				void queryClient.invalidateQueries({ queryKey: getListTasksQueryKey() });
				modal.resolve();
				void modal.hide();
			},
		},
	});

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		if (!title.trim()) {
			return;
		}

		createTask.mutate({
			data: {
				title: title.trim(),
				description: description.trim(),
			},
		});
	};

	return (
		<Dialog open={modal.visible} onOpenChange={(open) => !open && modal.hide()}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Create New Task</DialogTitle>
					<DialogDescription>
						Add a new task to your kanban board. It will start in the Todo column.
					</DialogDescription>
				</DialogHeader>

				<form onSubmit={handleSubmit} className="space-y-4">
					<div className="space-y-2">
						<label htmlFor="title" className="text-sm font-medium">
							Title
						</label>
						<Input
							id="title"
							value={title}
							onChange={(e) => setTitle(e.target.value)}
							placeholder="What needs to be done?"
							autoFocus
						/>
					</div>

					<div className="space-y-2">
						<label htmlFor="description" className="text-sm font-medium">
							Description
						</label>
						<Textarea
							id="description"
							value={description}
							onChange={(e) => setDescription(e.target.value)}
							placeholder="Describe the task in detail..."
							rows={4}
						/>
					</div>

					<DialogFooter>
						<Button type="button" variant="outline" onClick={() => modal.hide()}>
							Cancel
						</Button>
						<Button type="submit" disabled={!title.trim() || createTask.isPending}>
							{createTask.isPending ? "Creating..." : "Create Task"}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	);
});

export const CreateTaskDialog = defineModal<CreateTaskDialogProps, void>(CreateTaskDialogComponent);
