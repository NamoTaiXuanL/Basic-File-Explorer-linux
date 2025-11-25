using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;

class Program
{
    static void Main()
    {
        try
        {
            string pngPath = "material/png/logo_icon_0_150.png";
            string icoPath = "material/png/logo_icon_0_150.ico";

            // 确保输出目录存在
            Directory.CreateDirectory("material/png");

            using (Bitmap bmp = new Bitmap(pngPath))
            {
                // 创建包含多个尺寸的图标
                Icon icon = CreateIconFromBitmap(bmp);
                using (FileStream fs = new FileStream(icoPath, FileMode.Create))
                {
                    icon.Save(fs);
                }
            }

            Console.WriteLine("图标转换成功: " + icoPath);
        }
        catch (Exception ex)
        {
            Console.WriteLine("转换失败: " + ex.Message);
        }
    }

    static Icon CreateIconFromBitmap(Bitmap bmp)
    {
        // 创建包含16x16, 32x32, 48x48, 64x64, 128x128, 256x256尺寸的图标
        int[] sizes = { 16, 32, 48, 64, 128, 256 };

        // 使用32x32作为基础尺寸创建图标
        using (Bitmap iconBmp = new Bitmap(32, 32))
        {
            using (Graphics g = Graphics.FromImage(iconBmp))
            {
                g.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.HighQualityBicubic;
                g.DrawImage(bmp, 0, 0, 32, 32);
            }
            return Icon.FromHandle(iconBmp.GetHicon());
        }
    }
}