import { cn } from "@/lib/utils";
import { Card, CardContent, CardFooter, CardHeader } from "./card";
import { Avatar, AvatarFallback, AvatarImage } from "./avatar";
import { Button } from "./button";
import { MessageSquare } from "lucide-react";
import Link from "next/link";

interface PostCardProps {
  id: string;
  title: string;
  content: string;
  author: {
    id: string;
    name: string;
    image?: string;
  };
  commentCount: number;
  className?: string;
}

export function PostCard({
  id,
  title,
  content,
  author,
  commentCount,
  className,
}: PostCardProps) {
  return (
    <Card className={cn("w-full hover:shadow-md transition-shadow", className)}>
      <CardHeader className="flex flex-row items-center gap-4">
        <Avatar>
          <AvatarImage src={author.image} alt={author.name} />
          <AvatarFallback>
            {author.name.substring(0, 2).toUpperCase()}
          </AvatarFallback>
        </Avatar>
        <div className="flex flex-col">
          <Link href={`/posts/${id}`} className="font-semibold hover:underline">
            {title}
          </Link>
          <Link href={`/users/${author.id}`} className="text-sm text-muted-foreground hover:underline">
            by {author.name}
          </Link>
        </div>
      </CardHeader>
      <CardContent>
        <p className="line-clamp-3 text-muted-foreground">{content}</p>
      </CardContent>
      <CardFooter>
        <Button variant="ghost" size="sm" asChild>
          <Link href={`/posts/${id}`} className="flex items-center gap-1">
            <MessageSquare className="h-4 w-4" />
            <span>{commentCount} comment{commentCount !== 1 ? 's' : ''}</span>
          </Link>
        </Button>
      </CardFooter>
    </Card>
  );
}

