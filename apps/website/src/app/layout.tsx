import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "MovaPad | Your phone. Your touchpad.",
  description: "Turn your smartphone into a high-performance, zero-latency wireless touchpad for your Mac and PC.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${inter.className} bg-black text-white antialiased`}>{children}</body>
    </html>
  );
}
