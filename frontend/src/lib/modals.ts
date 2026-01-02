import type { NiceModalHocProps } from "@ebay/nice-modal-react";
import NiceModal from "@ebay/nice-modal-react";
import type React from "react";

export type NoProps = Record<string, never>;

type ComponentProps<P> = [P] extends [undefined] ? NoProps : P;
type ShowArgs<P> = [P] extends [undefined] ? [] : [props: P];

export type Modalized<P, R> = React.ComponentType<ComponentProps<P>> & {
	__modalResult?: R;
	show: (...args: ShowArgs<P>) => Promise<R>;
	hide: () => void;
	remove: () => void;
};

export function defineModal<P, R>(
	component: React.ComponentType<ComponentProps<P> & NiceModalHocProps>,
): Modalized<P, R> {
	const c = component as unknown as Modalized<P, R>;
	c.show = ((...args: ShowArgs<P>) =>
		NiceModal.show(
			component as React.FC<ComponentProps<P>>,
			args[0] as ComponentProps<P>,
		) as Promise<R>) as Modalized<P, R>["show"];
	c.hide = () => NiceModal.hide(component as React.FC<ComponentProps<P>>);
	c.remove = () => NiceModal.remove(component as React.FC<ComponentProps<P>>);
	return c;
}

export type ConfirmResult = "confirmed" | "canceled";
export type DeleteResult = "deleted" | "canceled";
export type SaveResult = "saved" | "canceled";
