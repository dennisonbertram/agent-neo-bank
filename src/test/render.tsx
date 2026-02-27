import { render } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import type { ReactElement } from "react";

interface RenderOptions {
  route?: string;
}

export function renderWithRouter(ui: ReactElement, options: RenderOptions = {}) {
  const { route = "/" } = options;
  return render(
    <MemoryRouter initialEntries={[route]}>
      {ui}
    </MemoryRouter>
  );
}
