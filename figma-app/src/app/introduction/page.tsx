"use client";

import { Button } from "@/components/ui/button";

import { toast } from "sonner";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "@/components/ui/drawer";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";
import * as React from "react";
import { SVGProps } from "react";
const Page = (props: SVGProps<SVGSVGElement>) => (
  <svg
    width={900}
    height={700}
    viewBox="0 0 900 700"
    xmlns="http://www.w3.org/2000/svg"
    {...props}
  >
    <rect width={900} height={700} fill="#f9fafc" rx={8} ry={8} />
    <rect x={0} y={0} width={900} height={80} fill="#2a3548" rx={8} ry={8} />
    <text
      x={40}
      y={50}
      fontSize={28}
      fontFamily="Arial"
      fill="white"
      fontWeight="bold"
    >
      {"Interactive Controls"}
    </text>
    <g
      id="notification-bell"
      transform="translate(800, 40)"
      onClick={(e) => {
        toast("New notification");
      }}
    >
      <path
        d="M0,0 C0,-8 -5,-15 -10,-15 C-15,-15 -20,-8 -20,0 L-20,10 L-16,10 L-16,15 L-4,15 L-4,10 L0,10 Z"
        fill="#ffd54f"
      />
      <circle cx={-10} cy={-18} r={4} fill="#ff5252" />
    </g>
    <g transform="translate(40, 120)">
      <rect
        x={0}
        y={0}
        width={380}
        height={240}
        fill="white"
        rx={8}
        ry={8}
        stroke="#e0e0e0"
        strokeWidth={1}
      />
      <text
        x={20}
        y={40}
        fontSize={22}
        fontFamily="Arial"
        fill="#2a3548"
        fontWeight="bold"
      >
        {"Basic UI Controls"}
      </text>
      <g transform="translate(30, 70)">
        <Dialog>
          <DialogTrigger asChild>
            <rect
              id="modal-button"
              x={0}
              y={0}
              width={140}
              height={50}
              rx={8}
              fill="#4361ee"
            />
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
        <text x={30} y={30} fontSize={16} fontFamily="Arial" fill="white">
          {"Open Modal"}
        </text>
      </g>
      <g transform="translate(200, 70)">
        <HoverCard openDelay={100} closeDelay={100}>
          <HoverCardTrigger asChild>
            <rect
              id="hover-card-button"
              x={0}
              y={0}
              width={140}
              height={50}
              rx={8}
              fill="#3a86ff"
            />
          </HoverCardTrigger>
          <HoverCardContent className="w-80 p-4">
            <div className="font-medium"></div>
            <p className="text-sm text-muted-foreground"></p>
            <div
              dangerouslySetInnerHTML={{
                __html: "Hover Card Content",
              }}
            ></div>
          </HoverCardContent>
        </HoverCard>
        <text x={25} y={30} fontSize={16} fontFamily="Arial" fill="white">
          {"Hover Card"}
        </text>
      </g>
      <g transform="translate(30, 150)">
        <div></div>
        <text x={55} y={30} fontSize={16} fontFamily="Arial" fill="white">
          {"Link"}
        </text>
      </g>
      <g transform="translate(200, 150)">
        <div></div>
        <text
          x={67}
          y={32}
          fontSize={24}
          fontFamily="Arial"
          fill="white"
          fontWeight="bold"
        >
          {"?"}
        </text>
      </g>
    </g>
    <g transform="translate(460, 120)">
      <rect
        x={0}
        y={0}
        width={380}
        height={240}
        fill="white"
        rx={8}
        ry={8}
        stroke="#e0e0e0"
        strokeWidth={1}
      />
      <text
        x={20}
        y={40}
        fontSize={22}
        fontFamily="Arial"
        fill="#2a3548"
        fontWeight="bold"
      >
        {"Navigation Controls"}
      </text>
      <Popover>
        <PopoverTrigger asChild>
          <g id="popover-button" transform="translate(30, 80)">
            <rect x={0} y={0} width={140} height={50} rx={8} fill="#7209b7" />
            <text x={40} y={30} fontSize={16} fontFamily="Arial" fill="white">
              {"Popover"}
            </text>
            <circle cx={120} cy={25} r={10} fill="white" />
            <rect x={115} y={20} width={10} height={2} fill="#7209b7" />
            <rect x={115} y={25} width={10} height={2} fill="#7209b7" />
            <rect x={115} y={30} width={10} height={2} fill="#7209b7" />
          </g>
        </PopoverTrigger>
        <PopoverContent className="w-80 p-4" align="bottom">
          <div className="font-medium pb-2"></div>
          <p className="text-sm text-muted-foreground"></p>
          <div
            dangerouslySetInnerHTML={{
              __html: "Popover Content",
            }}
          ></div>
        </PopoverContent>
      </Popover>
      <Sheet>
        <SheetTrigger asChild>
          <g id="sheet-button" transform="translate(200, 80)">
            <rect x={0} y={0} width={140} height={50} rx={8} fill="#b5179e" />
            <text x={30} y={30} fontSize={16} fontFamily="Arial" fill="white">
              {"Open Sheet"}
            </text>
            <path
              d="M125,20 L115,25 L125,30"
              fill="none"
              stroke="white"
              strokeWidth={2}
            />
          </g>
        </SheetTrigger>
        <SheetContent side="left">
          <SheetHeader>
            <SheetTitle></SheetTitle>
            <SheetDescription></SheetDescription>
          </SheetHeader>
          <div
            dangerouslySetInnerHTML={{
              __html: "Sheet Content",
            }}
          ></div>
        </SheetContent>
      </Sheet>
      <g transform="translate(115, 160)">
        <Drawer>
          <DrawerTrigger asChild>
            <rect
              id="drawer-button"
              x={0}
              y={0}
              width={140}
              height={50}
              rx={8}
              fill="#480ca8"
            />
          </DrawerTrigger>
          <DrawerContent>
            <DrawerHeader>
              <DrawerTitle></DrawerTitle>
              <DrawerDescription></DrawerDescription>
            </DrawerHeader>
            <DrawerFooter>
              <DrawerClose asChild>
                <Button variant="outline"></Button>
              </DrawerClose>
            </DrawerFooter>
          </DrawerContent>
        </Drawer>
        <text x={35} y={30} fontSize={16} fontFamily="Arial" fill="white">
          {"Drawer"}
        </text>
        <path
          d="M110,15 L120,25 L110,35"
          fill="none"
          stroke="white"
          strokeWidth={2}
        />
      </g>
    </g>
    <rect x={40} y={670} width={820} height={1} fill="#e0e0e0" />
    <text x={40} y={690} fontSize={14} fontFamily="Arial" fill="#6c757d">
      {"Interactive SVG Demo for the svg_react.rs script"}
    </text>
  </svg>
);
export default Page;
