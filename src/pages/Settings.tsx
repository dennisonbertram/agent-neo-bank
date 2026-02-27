import { InvitationCodes } from "./Settings/InvitationCodes";

export function Settings() {
  return (
    <div className="p-6 space-y-8">
      <h1 className="text-2xl font-bold">Settings</h1>
      <InvitationCodes />
    </div>
  );
}
