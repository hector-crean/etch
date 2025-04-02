import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import React from "react";
const Page = () => (
  <svg width="800" height="600" viewBox="0 0 800 600">
    <g transform="translate(50, 50)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <Dialog>
        <DialogTrigger asChild>
          <circle id="blue-circle" cx="50" cy="30" r="20" fill="#4a90e2" />
        </DialogTrigger>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle></DialogTitle>
            <DialogDescription></DialogDescription>
          </DialogHeader>
          <div
            dangerouslySetInnerHTML={{
              __html: "Modal Content",
            }}
          ></div>
        </DialogContent>
      </Dialog>
      <text x="80" y="35" font-size="12" font-family="Arial" />
      <rect
        id="close-button"
        x="0"
        y="60"
        width="80"
        height="30"
        rx="5"
        fill="#f44336"
      />
      <text x="15" y="80" fill="white" font-size="12" font-family="Arial" />
      <rect
        id="toggle-button"
        x="0"
        y="100"
        width="80"
        height="30"
        rx="5"
        fill="#673ab7"
      />
      <text x="15" y="120" fill="white" font-size="12" font-family="Arial" />
    </g>
    <g transform="translate(250, 50)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <g id="menu-icon" transform="translate(20, 20)">
        <rect x="0" y="0" width="30" height="5" rx="2" fill="#333" />
        <rect x="0" y="10" width="30" height="5" rx="2" fill="#333" />
        <rect x="0" y="20" width="30" height="5" rx="2" fill="#333" />
      </g>
      <text x="60" y="30" font-size="12" font-family="Arial" />
      <g id="sheet-close" transform="translate(20, 60)">
        <circle cx="15" cy="15" r="15" fill="#e91e63" />
        <line x1="8" y1="8" x2="22" y2="22" stroke="white" strokeWidth="2" />
        <line x1="8" y1="22" x2="22" y2="8" stroke="white" strokeWidth="2" />
      </g>
      <text x="45" y="80" font-size="12" font-family="Arial" />
    </g>
    <g transform="translate(450, 50)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <path
        id="notification-bell"
        d="M20,15 C20,10 15,5 10,5 C5,5 0,10 0,15 L0,25 L4,25 L4,30 L16,30 L16,25 L20,25 Z"
        transform="translate(15, 20)"
        fill="#ff9800"
      />
      <text x="50" y="35" fontSize="12" fontFamily="Arial" />
      <g id="error-icon" transform="translate(15, 60)">
        <circle cx="15" cy="15" r="15" fill="#f44336" />
        <text x="11" y="20" fill="white" font-size="16" font-family="Arial" />
      </g>
      <text x="50" y="80" font-size="12" font-family="Arial" />
    </g>
    <g transform="translate(50, 200)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <rect
        id="tab-item-1"
        x="0"
        y="15"
        width="100"
        height="30"
        rx="5"
        fill="#009688"
      />
      <text x="30" y="35" fill="white" font-size="12" font-family="Arial" />
    </g>
    <g transform="translate(250, 200)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <g id="faq-item" transform="translate(0, 15)">
        <rect x="0" y="0" width="180" height="40" rx="5" fill="#2196f3" />
        <text x="15" y="25" fill="white" font-size="12" font-family="Arial" />
        <text x="160" y="25" fill="white" font-size="16" font-family="Arial" />
      </g>
    </g>
    <g transform="translate(50, 300)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <rect
        id="dropdown-trigger"
        x="0"
        y="15"
        width="120"
        height="30"
        rx="5"
        fill="#3f51b5"
      />
      <text x="15" y="35" fill="white" font-size="12" font-family="Arial" />
      <rect
        id="dropdown-close"
        x="0"
        y="55"
        width="120"
        height="30"
        rx="5"
        fill="#9c27b0"
      />
      <text x="10" y="75" fill="white" font-size="12" font-family="Arial" />
    </g>
    <g transform="translate(250, 300)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <rect
        id="hover-area"
        x="0"
        y="15"
        width="150"
        height="60"
        rx="5"
        fill="#4caf50"
      />
      <text x="30" y="45" fill="white" font-size="12" font-family="Arial" />
    </g>
    <g transform="translate(450, 300)">
      <text x="0" y="0" font-size="16" font-family="Arial" />
      <rect
        id="input-field"
        x="0"
        y="15"
        width="150"
        height="40"
        rx="5"
        fill="white"
        stroke="#ccc"
        strokeWidth="1"
      />
      <text x="15" y="40" fill="#999" font-size="12" font-family="Arial" />
    </g>
    <g transform="translate(50, 450)">
      <rect
        x="0"
        y="0"
        width="700"
        height="100"
        rx="10"
        fill="#f5f5f5"
        stroke="#ddd"
      />
      <text x="20" y="30" font-size="14" font-family="Arial" fill="#333" />
      <text x="20" y="60" font-size="14" font-family="Arial" fill="#333" />
      <text x="20" y="90" font-size="14" font-family="Arial" fill="#333" />
    </g>
  </svg>
);
export default Page;
