import { cityTitle } from "../../../../lib/citySeo";

export async function generateMetadata() {
  return { title: cityTitle() };
}

export function generateStaticParams() {
  return [{ locale: "en", city: "vancouver" }];
}

export default function Page() {
  return null;
}
