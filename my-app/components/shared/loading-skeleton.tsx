import { Skeleton } from "@/components/ui/skeleton";
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card";

interface LoadingSkeletonProps {
  count?: number;
}

export function LoadingSkeleton({ count = 1 }: LoadingSkeletonProps) {
  return (
    <>
      {Array.from({ length: count }).map((_, index) => (
        <Card key={index} className="w-full">
          <CardHeader className="flex flex-row items-center gap-4">
            <Skeleton className="h-12 w-12 rounded-full" />
            <div className="space-y-2">
              <Skeleton className="h-4 w-[250px]" />
              <Skeleton className="h-4 w-[150px]" />
            </div>
          </CardHeader>
          <CardContent className="space-y-2">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-3/4" />
          </CardContent>
          <CardFooter>
            <Skeleton className="h-9 w-[120px]" />
          </CardFooter>
        </Card>
      ))}
    </>
  );
}

