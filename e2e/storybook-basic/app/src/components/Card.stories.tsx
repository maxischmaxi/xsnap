import type { Meta, StoryObj } from "@storybook/react";
import { Card } from "./Card";

const meta: Meta<typeof Card> = {
  title: "Components/Card",
  component: Card,
};

export default meta;
type Story = StoryObj<typeof Card>;

export const Default: Story = {
  args: {
    title: "Card Title",
    description: "This is a description of the card component with some example text.",
  },
};

export const WithAction: Story = {
  args: {
    title: "Card With Action",
    description: "This card has an action button at the bottom.",
    actionLabel: "Learn More",
  },
};
