import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import Link from "next/link";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: {
    template: "%s | Social Feed",
    default: "Social Feed - A minimal social media app",
  },
  description: "A minimal social media app built with Next.js and shadcn/ui",
  viewport: "width=device-width, initial-scale=1",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} min-h-screen bg-background antialiased`}
      >
        <header className="border-b">
          <div className="container mx-auto px-4 py-4 flex items-center justify-between">
            <Link href="/" className="text-xl font-bold">
              Social Feed
            </Link>
            <nav className="flex gap-4">
              <Link 
                href="/" 
                className="text-sm font-medium hover:text-primary"
              >
                Home
              </Link>
              <Link 
                href="/users/1" 
                className="text-sm font-medium hover:text-primary"
              >
                Profile Example
              </Link>
            </nav>
          </div>
        </header>
        
        <main className="flex-1">
          {children}
        </main>
        
        <footer className="border-t py-6 mt-8">
          <div className="container mx-auto px-4 text-center text-sm text-muted-foreground">
            <p>Social Feed - A minimal Next.js and shadcn/ui demo application</p>
          </div>
        </footer>
      </body>
    </html>
  );
}
