import { cn } from "@/lib/utils";
import { Card, CardContent, CardHeader } from "./card";
import { Avatar, AvatarFallback, AvatarImage } from "./avatar";
import { Badge } from "./badge";

interface UserInfoProps {
  user: {
    id: string;
    name: string;
    username: string;
    email: string;
    image?: string;
    bio?: string;
    postCount: number;
  };
  className?: string;
  followersCount?: number;
  followingCount?: number;
}

export function UserInfo({ user, className, followersCount, followingCount }: UserInfoProps) {
  return (
    <Card className={cn("w-full", className)}>
      <CardHeader className="flex flex-row items-center gap-4 pb-2">
        <Avatar className="h-16 w-16">
          <AvatarImage src={user.image} alt={user.name} />
          <AvatarFallback>{user.name.substring(0, 2).toUpperCase()}</AvatarFallback>
        </Avatar>
        <div className="flex flex-col">
          <h2 className="text-2xl font-bold">{user.name}</h2>
          <p className="text-muted-foreground">@{user.username}</p>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {user.bio && <p className="text-sm">{user.bio}</p>}
        <div className="flex flex-col space-y-1">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">Email:</span>
            <span className="text-sm text-muted-foreground">{user.email}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">Posts:</span>
            <Badge variant="secondary" className="text-xs">
              {user.postCount}
            </Badge>
          </div>
          {(followersCount !== undefined || followingCount !== undefined) && (
            <>
              {followersCount !== undefined && (
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">Followers:</span>
                  <Badge variant="secondary" className="text-xs">
                    {followersCount}
                  </Badge>
                </div>
              )}
              {followingCount !== undefined && (
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium">Following:</span>
                  <Badge variant="secondary" className="text-xs">
                    {followingCount}
                  </Badge>
                </div>
              )}
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

