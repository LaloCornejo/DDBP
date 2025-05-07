// Mock data for our application

// Users
export const users = [
  {
    id: "1",
    name: "John Doe",
    username: "johndoe",
    email: "john@example.com",
    image: "https://randomuser.me/api/portraits/men/1.jpg",
    bio: "Software engineer and tech enthusiast.",
    postCount: 2
  },
  {
    id: "2",
    name: "Jane Smith",
    username: "janesmith",
    email: "jane@example.com",
    image: "https://randomuser.me/api/portraits/women/2.jpg",
    bio: "UX designer and digital artist.",
    postCount: 1
  },
  {
    id: "3",
    name: "Alex Johnson",
    username: "alexj",
    email: "alex@example.com",
    image: "https://randomuser.me/api/portraits/men/3.jpg",
    bio: "Product manager and entrepreneur.",
    postCount: 1
  }
];

// Posts
export const posts = [
  {
    id: "1",
    title: "Getting Started with Next.js",
    content: "Next.js is a React framework that enables you to build server-side rendered applications. It offers a great developer experience with features like file-system based routing, API routes, and built-in CSS support. In this post, we'll explore how to get started with Next.js and create your first application. We'll cover the basic concepts, project structure, and deployment options. By the end of this tutorial, you'll have a solid understanding of how Next.js works and how it can benefit your projects.",
    authorId: "1",
    commentCount: 2,
    timestamp: "2025-04-30T12:00:00Z"
  },
  {
    id: "2",
    title: "The Power of Shadcn UI Components",
    content: "Shadcn UI is a collection of beautifully designed, accessible, and customizable components built on top of Tailwind CSS. In this post, we'll look at how to leverage these components to create stunning user interfaces quickly. We'll demonstrate how to install and configure the components, how to customize them to match your brand, and how to combine them to create complex UI patterns. By using a component library like Shadcn UI, you can save time and ensure consistency across your application.",
    authorId: "1",
    commentCount: 1,
    timestamp: "2025-05-01T15:30:00Z"
  },
  {
    id: "3",
    title: "Modern Web Design Trends in 2025",
    content: "Web design is constantly evolving, with new trends emerging every year. In 2025, we're seeing a shift towards more immersive experiences, minimalistic interfaces, and personalized content. This post explores the top design trends that are shaping the web this year. From neomorphism to micro-interactions, from dark mode to voice user interfaces, we'll examine what makes these trends effective and how you can incorporate them into your projects.",
    authorId: "2",
    commentCount: 3,
    timestamp: "2025-05-02T09:15:00Z"
  },
  {
    id: "4",
    title: "Building a Scalable API with Node.js",
    content: "Creating a scalable and maintainable API is essential for modern web applications. In this comprehensive guide, we'll walk through the process of building a robust API using Node.js and Express. We'll cover topics such as API design principles, authentication, error handling, testing, and performance optimization. By the end of this tutorial, you'll have the knowledge to create APIs that can handle high traffic and scale with your application's needs.",
    authorId: "3",
    commentCount: 0,
    timestamp: "2025-05-03T18:45:00Z"
  }
];

// Comments
export const comments = [
  {
    id: "1",
    postId: "1",
    content: "Great introduction to Next.js! I've been looking for a comprehensive guide like this.",
    authorId: "2",
    timestamp: "2025-04-30T14:23:00Z"
  },
  {
    id: "2",
    postId: "1",
    content: "Thanks for sharing this. I'm just getting started with Next.js and found this really helpful.",
    authorId: "3",
    timestamp: "2025-04-30T18:15:00Z"
  },
  {
    id: "3",
    postId: "2",
    content: "I've been using Shadcn UI for my recent project and it's been a game changer. The components are so well designed!",
    authorId: "2",
    timestamp: "2025-05-01T16:42:00Z"
  },
  {
    id: "4",
    postId: "3",
    content: "Interesting insights on the design trends. I've noticed the shift toward minimalism as well.",
    authorId: "1",
    timestamp: "2025-05-02T10:30:00Z"
  },
  {
    id: "5",
    postId: "3",
    content: "Voice user interfaces are definitely going to be huge this year. Great analysis!",
    authorId: "3",
    timestamp: "2025-05-02T12:18:00Z"
  },
  {
    id: "6",
    postId: "3",
    content: "I'm excited to implement some of these trends in my upcoming projects.",
    authorId: "1",
    timestamp: "2025-05-02T15:45:00Z"
  }
];

// Helper functions to simulate API operations
export function getUserById(id: string) {
  return users.find(user => user.id === id);
}

export async function getPostById(id: string) {
  return posts.find(post => post.id === id);
}

export function getCommentsByPostId(postId: string) {
  return comments.filter(comment => comment.postId === postId);
}

export function getPostsByAuthorId(authorId: string) {
  return posts.filter(post => post.authorId === authorId);
}

export function getPaginatedPosts(page: number = 1, limit: number = 10) {
  const startIndex = (page - 1) * limit;
  const endIndex = page * limit;
  
  return {
    posts: posts
      .slice(startIndex, endIndex)
      .map(post => {
        const author = getUserById(post.authorId);
        return {
          ...post,
          author: {
            id: author?.id,
            name: author?.name,
            image: author?.image
          }
        };
      }),
    hasMore: endIndex < posts.length
  };
}

export async function enrichPostWithDetails(post: typeof posts[0]) {
  const author = getUserById(post.authorId);
  const postComments = getCommentsByPostId(post.id).map(comment => {
    const commentAuthor = getUserById(comment.authorId);
    return {
      ...comment,
      author: {
        id: commentAuthor?.id,
        name: commentAuthor?.name,
        image: commentAuthor?.image
      }
    };
  });
  
  return {
    ...post,
    author: {
      id: author?.id,
      name: author?.name,
      image: author?.image
    },
    comments: postComments
  };
}

