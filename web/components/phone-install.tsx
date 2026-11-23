import Link from "next/link";

/** Install stays one tap away on phones, where the header folds its links into a menu. */
export function PhoneInstall() {
  return (
    <Link
      href="/setup"
      className="ms-auto rounded-md border px-2.5 py-1 text-sm font-medium transition-colors hover:bg-fd-accent lg:hidden"
    >
      Install
    </Link>
  );
}
