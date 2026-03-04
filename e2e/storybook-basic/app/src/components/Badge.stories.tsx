import type { Meta, StoryObj } from "@storybook/react";
import { Badge } from "./Badge";

const meta: Meta<typeof Badge> = {
  title: "Components/Badge",
  component: Badge,
};

export default meta;
type Story = StoryObj<typeof Badge>;

export const Success: Story = {
  args: { label: "Success", variant: "success" },
};

export const Warning: Story = {
  args: { label: "Warning", variant: "warning" },
};

export const Error: Story = {
  args: { label: "Error", variant: "error" },
};

export const Info: Story = {
  args: { label: "Info", variant: "info" },
};
