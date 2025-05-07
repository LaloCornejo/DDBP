import { LoadingSkeleton } from "@/components/shared/loading-skeleton";

export default function Loading() {
  return (
    <div className="container mx-auto px-4 py-8" aria-live="polite" aria-busy="true">
      <div className="mb-8">
        <div className="h-8 w-48 bg-gray-200 rounded animate-pulse mb-2" aria-hidden="true"></div>
        <div className="h-4 w-96 bg-gray-200 rounded animate-pulse" aria-hidden="true"></div>
      </div>
      
      <div aria-label="Loading content">
        <LoadingSkeleton count={5} />
      </div>
      
      <div className="sr-only">Loading page content, please wait</div>
    </div>
  );
}

