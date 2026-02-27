import { test, expect } from "./fixtures";

test.describe("Onboarding flow", () => {
  test("renders Welcome step on /onboarding", async ({ page, mockTauri }) => {
    await mockTauri({});
    await page.goto("/onboarding");
    await expect(page.getByRole("heading", { name: "Welcome to Agent Neo Bank" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Get Started" })).toBeVisible();
  });

  test("navigates from Welcome to Email step", async ({ page, mockTauri }) => {
    await mockTauri({});
    await page.goto("/onboarding");
    await page.getByRole("button", { name: "Get Started" }).click();
    await expect(page.getByRole("heading", { name: "Enter your email" })).toBeVisible();
    await expect(page.getByPlaceholder("Email address")).toBeVisible();
  });

  test("Email step validates input and advances to OTP step", async ({
    page,
    mockTauri,
  }) => {
    await mockTauri({});
    await page.goto("/onboarding");

    // Go to email step
    await page.getByRole("button", { name: "Get Started" }).click();

    // Submit empty -> validation error
    await page.getByRole("button", { name: "Continue" }).click();
    await expect(page.getByText("Please enter a valid email")).toBeVisible();

    // Enter valid email and submit
    await page.getByPlaceholder("Email address").fill("test@example.com");
    await page.getByRole("button", { name: "Continue" }).click();

    // Should now be on OTP step
    await expect(
      page.getByRole("heading", { name: "Enter verification code" })
    ).toBeVisible();
  });
});
