import { Trans } from "react-i18next";
import { Module } from "./types";

export const landingData: Module = {
  meta: {
    title: <Trans i18nKey="general.home" />,
    module: null,
    link: "/",
  },
  sections: [
    {
      blocks: [
        {
          id: "96710fd7-8da9-447a-a855-f059d1a0a44b",
          type: "Aside",
          props: {
            showProgress: false,
            main: {
              id: "baca4b0c-fb3a-45ec-9086-23a88s220d05",
              type: "Landing",
              props: {
                url: "Mez105 Radiesse Hd 240612B.mp4",
                title: <Trans i18nKey="landing.main.title" />,
                muted: false,
                caption: null,
                poster: "moa_poster.jpg",
                hasAudioTrack: true,
                banner: {
                  title: <Trans i18nKey="landing.main.banner.title" />,
                  caption: <Trans i18nKey="landing.main.banner.caption" />,
                  backgroundImage: "banner_bg.png",
                },
                tiles: {
                  0: {
                    image: "tile_01.jpg",
                    icon: "icon_01.svg",
                  },
                  1: {
                    image: "tile_02.jpg",
                    icon: "icon_02.svg",
                  },
                  2: {
                    image: "tile_03.jpg",
                    icon: "icon_03.svg",
                  },
                },
                media: {
                  audio: "background_music.mp3",
                  video: "intro_animation.webm",
                  documents: [
                    "user_manual.pdf",
                    "quick_guide.doc",
                  ],
                },
              },
            },
            aside: {
              sectionIdx: 0,
              buttons: [],
              title: {
                type: "PlainText",
                id: "f4e62658-95a4-4cef-bb6d-432af64ba83e",
                props: {
                  text: <Trans i18nKey="landing.aside.title" />,
                  icon: "title_icon.png",
                },
              },
              help: {
                type: "HtmlText",
                id: "00dcda14-fe7b-4cd5-b781-453a9d62b9a1",
                props: {
                  html: <Trans i18nKey="landing.aside.help" />,
                  backgroundImage: "help_bg.jpg",
                },
              },
              blocks: [
                {
                  id: "e80dfdb8-f630-47ef-8165-fcd75f87e397",
                  type: "HtmlText",
                  props: {
                    html: <Trans i18nKey="landing.aside.body" />,
                    media: "aside_video.mp4",
                    thumbnail: "aside_thumb.webp",
                  },
                },
              ],
            },
          },
        },
      ],
    },
  ],
  assets: {
    gallery: [
      "gallery_01.jpg",
      "gallery_02.png",
      "gallery_03.gif",
    ],
    downloads: {
      manual: "product_manual.pdf",
      brochure: "marketing_brochure.pdf",
    },
    overlays: {
      loading: "loading_spinner.svg",
      success: "success_checkmark.png",
      error: "error_warning.ico",
    },
  },
}; 