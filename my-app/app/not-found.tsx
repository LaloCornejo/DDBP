import Link from "next/link";

export default function NotFound() {
  return (
    <div className="container mx-auto px-4 py-16 text-center">
      <h2 className="text-4xl font-bold mb-4">404</h2>
      <p className="text-xl mb-8">Page not found</p>
      <p className="text-muted-foreground mb-8">The page you're looking for doesn't exist or has been removed.</p>
      <Link 
        href="/" 
        className="inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90"
      >
        Go back home
      </Link>
    </div>
  );
}

