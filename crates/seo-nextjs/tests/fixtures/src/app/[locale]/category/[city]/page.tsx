export async function generateMetadata() {
  return { title: "City" };
}

export function generateStaticParams() {
  return [{ locale: "en", city: "vancouver" }];
}

export default function Page() {
  return null;
}
